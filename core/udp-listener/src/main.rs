use anyhow::anyhow;
use db_api::{place_client_order, run_migrations, ClientOrder};
use std::{env, io};
use tokio::net::UdpSocket;

struct Server {
    pool: sqlx::PgPool,
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            pool,
            socket,
            mut buf,
        } = self;

        loop {
            let metadata = Some(socket.recv_from(&mut buf).await?);
            let Some((length, _)) = metadata else {
                continue;
            };
            let data = &buf[..length];
            let raw_xml = String::from_utf8_lossy(data);

            tracing::info!("Received {length} bytes");

            let orders =
                match serde_xml_rs::from_str::<Vec<ClientOrder>>(&raw_xml) {
                    Ok(vec) => vec,
                    Err(e) => {
                        tracing::error!("Error parsing XML: {e}");
                        continue;
                    }
                };

            tracing::info!("Parsed {:#?} orders", orders.len());

            for order in orders.iter() {
                match place_client_order(&pool, order).await {
                    Ok(_) => tracing::info!("Order successfully placed"),
                    Err(e) => tracing::error!("Error placing order: {e}"),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => return Err(anyhow!(e)),
    };
    tracing::info!("Connecting to database...");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    if let Err(e) = run_migrations(&pool).await {
        tracing::error!("Error running migrations: {e}");
        return Err(anyhow!(e));
    }

    tracing::info!("DB connection and initializtion successfull.");

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    tracing::info!("Listening on: {}", socket.local_addr()?);

    //TODO: Ask what are the expected file sizes
    let server = Server {
        pool,
        socket,
        buf: vec![0; 10024],
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}
