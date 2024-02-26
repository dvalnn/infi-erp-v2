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
        tracing::info!("Generating BOM entry for new order {}", new_order_idx);
        let order = db_api::get_order(new_order_idx, &self.pool).await?;
        println!("Order: {:?}", order);

        Ok(())
    }
}
