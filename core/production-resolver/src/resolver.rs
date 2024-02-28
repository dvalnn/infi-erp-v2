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

    pub async fn generate_bom_entries(&self, order_id: i64) -> Result<(), anyhow::Error> {
        tracing::info!("Generating BOM entry for order with id {}", order_id);
        let order = db_api::get_order(order_id, &self.pool).await?;
        let recipe = db_api::get_repice_to_root(order.piece_id, &self.pool).await?;

        tracing::debug!("Recipe for order with id {}: {:#?}", order_id, recipe);

        //NOTE: A graph may be a better representation for the recipe.
        //      Since the decision algorithm is still very simple, this
        //      approach is sufficient for now
        let recipe_map = Resolver::map_flat_recipe(recipe);

        // Decide on the path to take. For now, just take the least cost path.
        // Later, we may want to take into account the availability of the tools
        // and maybe real time data from the MES.
        let bom_entries = get_cheapest_path(order.piece_id, recipe_map);

        let steps_total = bom_entries.len() as i32;
        let pieces_total = order.quantity;

        for piece_number in 1..=pieces_total {
            for (step_number, transformation) in bom_entries.iter().rev().enumerate() {
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

fn get_cheapest_path(starting_piece: i64, recipe_map: HashMap<i64, Recipe>) -> Vec<Transformation> {
    let mut bom_entries = Vec::new();

    let mut current_piece = starting_piece;
    loop {
        let Some(available_paths) = recipe_map.get(&current_piece) else {
            break;
        };

        if let Some(cheapest_path) = available_paths.iter().min_by_key(|t| t.cost.0) {
            current_piece = cheapest_path.from_piece;
            bom_entries.push(cheapest_path.clone());
        }
    }
    bom_entries
}

#[cfg(test)]
mod tests {
    use super::*;
    // use db_api::{Tools, Transformation};
    // use sqlx::postgres::types::PgMoney;

    #[test]
    fn test_map_flat_recipe() {
        todo!("Write test for Resolver::map_flat_recipe")
    }
}
