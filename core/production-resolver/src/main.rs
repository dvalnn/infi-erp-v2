use anyhow::anyhow;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

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
    let mut notification_listener =
        sqlx::postgres::PgListener::connect(&database_url).await?;

    let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
    notification_listener
        .listen(db_api::ORDER_NOTIFY_CHANNEL)
        .await?;

    tracing::info!("DB connection and initializtion successfull.");

    loop {
        let notification = match notification_listener.recv().await {
            Ok(notif) => notif,
            Err(e) => return Err(anyhow!(e)),
        };
        println!("Received notification: {:#?}", notification);
    }
}
