use std::collections::HashMap;

use db_api::{Bom, Order, Recipe, Transformation};
use sqlx::{postgres::PgListener, PgPool};

pub struct Resolver {
    pool: PgPool,
    listener: PgListener,
}

impl Resolver {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    pub async fn handle_new_order(
        &self,
        payload: &str,
    ) -> Result<(), anyhow::Error> {
        let order_id: i64 = payload.parse()?;
        tracing::info!("Received new order with id {}", order_id);
        self.generate_bom_entries(order_id).await?;
        Ok(())
    }

    pub async fn handle_new_bom_entry(
        &self,
        payload: &str,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("Received new bom entries {}", payload);

        let ids = payload
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect::<Vec<i64>>();

        if ids.is_empty() {
            return Err(anyhow::anyhow!("Invalid payload: {}", payload));
        }

        tracing::debug!("Parsed new bom entries {:#?}", ids);

        let entries = ids.iter().fold(Vec::new(), |mut acc, id| {
            acc.push(Bom::get_by_id(*id, &self.pool));
            acc
        });

        let entries = futures::future::try_join_all(entries).await?;

        //TODO: query the MES and check which lines are compatible with
        //      the new bom entries. If no line is compatible, log a warning

        let order = db_api::get_order(entries[0].order_id, &self.pool).await?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        // TODO: lauch a task to generate entries for existing orders
        // should be done in a separate task to avoid blocking the main loop
        // tokio::spawn(...)

        //start listening on the notification channels
        use db_api::NotificationChannel as Nc;
        self.listener.listen_all(Nc::ALL_STR).await?;

        // TODO: handle and log errors instead of returning them
        loop {
            let notification = self.listener.recv().await?;
            let payload = notification.payload();
            let channel = Nc::from(notification.channel());

            match channel {
                Nc::NewOrder => {
                    self.handle_new_order(payload).await?;
                }
                Nc::NewBomEntry => {
                    self.handle_new_bom_entry(payload).await?;
                }
                Nc::Unknown => {
                    tracing::warn!(
                        "Received notification on unknown channel: {:#?}",
                        notification
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn calculate_ideal_prod_plan(
        order: &Order,
        bom: &mut [Bom],
    ) -> Result<(), anyhow::Error> {
        let deadline = order.due_date;
        let max_concurrent_orders = 3; // TODO: get this from MES or config

        // Group the BOM entries by their stage in the production process
        // of the final piece.
        bom.sort_by(|a, b| a.step_number.cmp(&b.step_number));

        // Calculate the ideal production plan
        //
        // The plan is a list of steps that must be taken to produce
        // the pieces in the order. The steps are grouped by the stage
        // in the production process of the final piece.
        //
        // The plan is calculated based on the due date of the order,
        // and the times it takes to produce each piece in the order.
        //
        // The plan is calculated in such a way that the production
        // process is as efficient as possible, and that the pieces
        // are ready to be shipped by the due date.

        //run back from the due date and last stages and assign bom entries
        //production time slots

        //NOTE: for now lets try to complete the order a day before the due date
        for day in (1..=deadline - 1).rev() {
            for timeslot in (1..=12).rev() {
                todo!("assing bom entries to production time slots");
            }
        }
        todo!("Calculate the ideal production plan");
    }

    /// Explodes a flat recipe into multiple paths.
    /// Assumes the recipe is flat and has no cycles.
    /// Also assumes that the recipe is ordered from the final
    /// to the root piece.
    fn map_flat_recipe(recipe: Recipe) -> HashMap<i64, Recipe> {
        let mut exploded = HashMap::new();

        for transformation in recipe {
            let to_piece = transformation.to_piece;
            let path = exploded.entry(to_piece).or_insert_with(Vec::new);
            path.push(transformation);
        }

        tracing::debug!("Exploded recipe: {:#?}", exploded);

        exploded
    }

    /// Decides the cheapest path to take to produce a piece.
    /// Assumes the recipe is has no cycles.
    /// Resulting vector is ordered from the given starting piece
    /// to the oposite end of the recipe.
    ///
    /// # Arguments
    /// * `starting_piece` - The piece to be produced
    /// * `recipe_map` - A map of pieces to the transformations that produce them
    ///                  The root piece should not be present in the map as it
    ///                  is not product of any transformation.
    ///
    /// The transformations are ordered based on their cost.
    fn get_cheapest_path(
        starting_piece: i64,
        recipe_map: HashMap<i64, Recipe>,
    ) -> Vec<Transformation> {
        let mut bom_entries = Vec::new();

        let mut current_piece = starting_piece;
        loop {
            let Some(available_paths) = recipe_map.get(&current_piece) else {
                break;
            };

            if let Some(cheapest_path) =
                available_paths.iter().min_by_key(|t| t.cost.0)
            {
                current_piece = cheapest_path.from_piece;
                bom_entries.push(cheapest_path.clone());
            }
        }
        bom_entries
    }

    /// Generates BOM entries for a given order.
    /// Will generate one entry for each piece to be produced.
    /// If the order is for 10 pieces, 10 entries will be generated
    /// and each entry will have the `piece_number` field set to its
    /// index in the range 1..=10.
    ///
    /// The BOM entries are generated based on the recipe for the
    /// piece to be produced. The recipe is a list of transformations
    /// that must be applied to the raw material to produce the final
    /// piece.
    ///
    /// The final piece must be a valid product and be present in the
    /// database of piece types along with its associated transformations.
    ///
    /// # Arguments
    /// * `order_id` - The id of the order for which to generate the BOM entries
    pub async fn generate_bom_entries(
        &self,
        order_id: i64,
    ) -> Result<(), anyhow::Error> {
        tracing::debug!("Starting BOM resolution for order {}", order_id);

        let order = db_api::get_order(order_id, &self.pool).await?;
        let recipe =
            db_api::get_repice_to_root(order.piece_id, &self.pool).await?;

        tracing::debug!("Recipe for order with id {}: {:#?}", order_id, recipe);

        //NOTE: A graph may be a better representation for the recipe.
        //      Since the decision algorithm is still very simple, this
        //      approach is sufficient for now
        let recipe_map = Resolver::map_flat_recipe(recipe);

        // Decide on the path to take. For now, just take the least cost path.
        // Later, we may want to take into account the availability of the tools
        // and maybe real time data from the MES.
        let bom_entries = Self::get_cheapest_path(order.piece_id, recipe_map);

        let steps_total = bom_entries.len() as i32;
        let pieces_total = order.quantity;

        let mut batch = Vec::new();
        for piece_number in 1..=pieces_total {
            for (step_number, transformation) in
                bom_entries.iter().rev().enumerate()
            {
                let bom = Bom::new(
                    order_id,
                    transformation.id,
                    piece_number,
                    pieces_total,
                    (step_number + 1) as i32,
                    steps_total,
                );
                batch.push(bom);
            }
        }

        Bom::insert_batch(&batch, &self.pool).await?;
        tracing::info!("BOM entries generated for order {}", order_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_api::{Tools, Transformation};
    use sqlx::postgres::types::PgMoney;

    static RECIPE: [Transformation; 4] = [
        Transformation {
            id: 1,
            from_piece: 1,
            to_piece: 2,
            cost: PgMoney(1),
            tool: Tools::T1,
            quantity: 1,
        },
        Transformation {
            id: 2,
            from_piece: 2,
            to_piece: 5,
            cost: PgMoney(100),
            tool: Tools::T2,
            quantity: 1,
        },
        Transformation {
            id: 3,
            from_piece: 2,
            to_piece: 5,
            cost: PgMoney(50),
            tool: Tools::T3,
            quantity: 1,
        },
        Transformation {
            id: 4,
            from_piece: 5,
            to_piece: 9,
            cost: PgMoney(100),
            tool: Tools::T2,
            quantity: 1,
        },
    ];

    #[test]
    fn test_map_flat_recipe() {
        let mut expected = HashMap::new();
        expected.insert(2, vec![RECIPE[0].clone()]);
        expected.insert(5, vec![RECIPE[1].clone(), RECIPE[2].clone()]);
        expected.insert(9, vec![RECIPE[3].clone()]);

        let result = Resolver::map_flat_recipe(RECIPE.to_vec());

        assert_eq!(expected, result);
    }

    #[test]
    fn test_get_cheapest_path() {
        let mut map = HashMap::new();
        map.insert(2, vec![RECIPE[0].clone()]);
        map.insert(5, vec![RECIPE[1].clone(), RECIPE[2].clone()]);
        map.insert(9, vec![RECIPE[3].clone()]);

        let result = get_cheapest_path(9, map);
        let expected =
            vec![RECIPE[3].clone(), RECIPE[2].clone(), RECIPE[0].clone()];
        assert_eq!(expected, result);
    }
}
