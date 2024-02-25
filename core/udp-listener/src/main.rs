use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{types::PgMoney, PgQueryResult},
    PgPool,
};
use std::{env, error::Error, io};
use tokio::net::UdpSocket;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
enum WorkPieces {
    P5,
    P6,
    P7,
    P9,
}

impl std::fmt::Display for WorkPieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkPieces::P5 => write!(f, "P5"),
            WorkPieces::P6 => write!(f, "P6"),
            WorkPieces::P7 => write!(f, "P7"),
            WorkPieces::P9 => write!(f, "P9"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Order {
    number: i32,
    work_piece: WorkPieces,
    quantity: i32,
    due_date: i32,
    late_pen: String,
    early_pen: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Client {
    name_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct ClientOrder {
    client: Client,
    order: Order,
}

struct Server {
    pool: sqlx::PgPool,
    socket: UdpSocket,
    buf: Vec<u8>,
}

type MyExecutor<'this> = &'this mut sqlx::PgConnection;

async fn place_new_order(
    pool: &PgPool,
    order: &ClientOrder,
) -> Result<PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let piece_id = sqlx::query!(
        "SELECT id FROM pieces WHERE name = $1 ",
        order.order.work_piece.to_string()
    )
    .fetch_one(&mut tx as MyExecutor)
    .await?
    .id;

    let client_id = sqlx::query!(
        "SELECT id FROM clients WHERE name = $1",
        order.client.name_id
    )
    .fetch_one(&mut tx as MyExecutor)
    .await;

    let client_id = match client_id {
        Ok(rec) => rec.id,
        Err(_) => {
            sqlx::query!(
                "INSERT INTO clients(name) VALUES($1) RETURNING id",
                order.client.name_id
            )
            .fetch_one(&mut tx as MyExecutor)
            .await?
            .id
        }
    };

    let result = sqlx::query!(
        "INSERT INTO orders (
            work_piece,
            client_id,
            order_number,
            quantity,
            due_date,
            late_penalty,
            early_penalty)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7);
        ",
        piece_id,
        client_id,
        order.order.number,
        order.order.quantity,
        order.order.due_date,
        PgMoney(3),
        PgMoney(4),
        // PgMoney(order.order.late_pen),
        // PgMoney(order.order.early_pen)
    )
    .execute(&mut tx as MyExecutor)
    .await?;

    tx.commit().await?;

    Ok(result)
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            pool,
            socket,
            mut buf,
        } = self;

        loop {
            // next message we're going to echo back.
            let metadata = Some(socket.recv_from(&mut buf).await?);
            let Some((length, _)) = metadata else {
                continue;
            };
            let data = &buf[..length];
            let raw_xml = String::from_utf8_lossy(data);

            let orders =
                match serde_xml_rs::from_str::<Vec<ClientOrder>>(&raw_xml) {
                    Ok(vec) => vec,
                    Err(e) => {
                        tracing::error!("Error parsing XML: {e}");
                        continue;
                    }
                };
            tracing::info!("Parsed {:#?} orders", orders.len());
            match place_new_order(&pool, &orders[0]).await {
                Ok(_) => tracing::info!("Order successfully placed"),
                Err(e) => tracing::error!("Error placing order: {e}"),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;

    let pool = sqlx::PgPool::connect(&database_url).await?;

    tracing_subscriber::fmt::init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    tracing::info!("Listening on: {}", socket.local_addr()?);

    //TODO: Ask what are the expected file sizes
    let server = Server {
        pool,
        socket,
        buf: vec![0; 1024],
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}
