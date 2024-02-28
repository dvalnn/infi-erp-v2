use std::collections::HashMap;

use db_api::{Recipe, Transformation};
use sqlx::{postgres::PgListener, PgPool};

pub struct Resolver {
    pool: PgPool,
    listener: PgListener,
}

impl Resolver {
    pub fn new(pool: PgPool, listener: PgListener) -> Self {
        Self { pool, listener }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        // TODO: lauch a task to generate entries for existing orders
        // should be done in a separate task to avoid blocking the main loop
        // tokio::spawn(...)

        loop {
            let notification = self.listener.recv().await?;
            let new_order_idx: i64 = notification.payload().parse()?;
            self.generate_bom_entries(new_order_idx).await?;
        }
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
        tracing::info!("Generating BOM entry for order with id {}", order_id);
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

        for piece_number in 1..=pieces_total {
            for (step_number, transformation) in
                bom_entries.iter().rev().enumerate()
            {
                let bom = db_api::Bom::new(
                    order_id,
                    transformation.id,
                    piece_number,
                    pieces_total,
                    (step_number + 1) as i32,
                    steps_total,
                );
                let id = bom.insert(&self.pool).await?;
                tracing::info!("Inserted BOM entry with id {}", id);
            }
        }

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
