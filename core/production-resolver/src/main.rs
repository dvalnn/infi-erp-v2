#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")?;
    tracing::info!("Connecting to database...");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    if let Err(e) = db_api::run_migrations(&pool).await {
        tracing::error!("Error running migrations: {e}");
        return Err(e);
    }
    let mut notification_listener =
        sqlx::postgres::PgListener::connect(&database_url).await?;

    notification_listener
        .listen(db_api::ORDER_NOTIFY_CHANNEL)
        .await?;

    tracing::info!("DB connection and initializtion successfull.");

    loop {
        let notification = notification_listener.recv().await?;
        println!("Received notification: {:#?}", notification);
    }
}
