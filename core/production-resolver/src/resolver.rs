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
            self.generate_bom_entry(new_order_idx).await?;
        }
    }

    pub async fn generate_bom_entry(
        &self,
        new_order_idx: i64,
    ) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Generating BOM entry for order with id {}",
            new_order_idx
        );
        let order = db_api::get_order(new_order_idx, &self.pool).await?;
        let recipe =
            db_api::get_repice_to_root(order.piece_id, &self.pool).await?;

        println!("Recipe for order with id {}: {:#?}", new_order_idx, recipe);

        // ... generate BOM entry

        Ok(())
    }
}
