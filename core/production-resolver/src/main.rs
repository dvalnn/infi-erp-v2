mod resolver;

use anyhow::anyhow;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => return Err(anyhow!(e)),
    };
    tracing::info!("Connecting to database...");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    if let Err(e) = db_api::run_migrations(&pool).await {
        tracing::error!("Error running migrations: {e}");
        return Err(anyhow!(e));
    }
    let notification_listener =
        sqlx::postgres::PgListener::connect(&database_url).await?;

    let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
    tracing::info!("DB connection and initializtion successfull.");

    let mut resolver = resolver::Resolver::new(pool, notification_listener);

    resolver.run().await?;

    Ok(())
}
