use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, postgres::types::PgMoney, PgPool};

pub const ORDER_NOTIFY_CHANNEL: &str = "new_order";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum WorkPieces {
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
pub struct Order {
    pub number: i32,
    pub work_piece: WorkPieces,
    pub quantity: i32,
    pub due_date: i32,
    pub late_pen: String,
    pub early_pen: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Client {
    pub name_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct ClientOrder {
    pub client: Client,
    pub order: Order,
}

#[derive(Debug)]
struct NewOrder {
    piece_id: i64,
    client_id: i64,
    number: i32,
    quantity: i32,
    due_date: i32,
    late_pen: i64,
    early_pen: i64,
}

/// Place a new order for a client with the given order details.
/// If the client does not exist, it will be created.
/// Money strings are expected to be in the format "$123.45" or "123.45€"
pub async fn place_client_order(
    pool: &PgPool,
    ClientOrder { client, order }: &ClientOrder,
) -> Result<i64, BoxDynError> {
    let late_penalty = parse_money_string(&order.late_pen)?;
    let early_penalty = parse_money_string(&order.early_pen)?;

    let mut tx = pool.begin().await?;
    let piece_id = get_piece_id(&order.work_piece, &mut tx).await?;
    let client_id = match get_client_id(client, &mut tx).await {
        Ok(id) => {
            tracing::info!("Client found! ID: {}", id);
            id
        }
        Err(_) => {
            tracing::info!("Client not found! Creating new client");
            sqlx::query!(
                "INSERT INTO clients(name) VALUES($1) RETURNING id",
                client.name_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        }
    };

    let order = NewOrder {
        piece_id,
        client_id,
        number: order.number,
        quantity: order.quantity,
        due_date: order.due_date,
        late_pen: late_penalty,
        early_pen: early_penalty,
    };

    tracing::info!("Placing order: {:#?}", order);
    let order_id = match place_new_order(order, &mut tx).await {
        Ok(id) => id,
        Err(e) => return Err(e.into()),
    };

    let query = format!("NOTIFY {}, '{}'", ORDER_NOTIFY_CHANNEL, order_id);
    sqlx::query(&query).execute(&mut *tx).await?;
    tx.commit().await?;

    Ok(order_id)
}

async fn place_new_order(
    order: NewOrder,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query!(
        "INSERT INTO orders (
            work_piece,
            client_id,
            order_number,
            quantity,
            due_date,
            late_penalty,
            early_penalty
        )
        VALUES
            ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        ",
        order.piece_id,
        order.client_id,
        order.number,
        order.quantity,
        order.due_date,
        PgMoney(order.late_pen),
        PgMoney(order.early_pen)
    )
    .fetch_one(&mut **tx)
    .await?
    .id)
}

pub async fn get_client_id(
    client: &Client,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query!("SELECT id FROM clients WHERE name = $1", client.name_id)
            .fetch_one(&mut **tx)
            .await?
            .id,
    )
}

pub async fn get_piece_id(
    piece_name: &WorkPieces,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query!(
        "SELECT id FROM pieces WHERE name = $1",
        piece_name.to_string()
    )
    .fetch_one(&mut **tx)
    .await?
    .id)
}

/// Parse a money string into a number of cents.
/// Non-digit characters are ignored.
/// This function assumes that the input is a valid money string.
///
/// # Examples:
/// ```
/// use db_api::parse_money_string;
/// let money = "$123.45";
/// assert_eq!(parse_money_string(money), Ok(12345));
///
/// let money = "123.45€";
/// assert_eq!(parse_money_string(money), Ok(12345));
/// ```
pub fn parse_money_string(money: &str) -> Result<i64, std::num::ParseIntError> {
    money
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
}

pub async fn run_migrations(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_money_string() {
        let money = "$123.45";
        assert_eq!(parse_money_string(money), Ok(12345));
        let money = "123.45€";
        assert_eq!(parse_money_string(money), Ok(12345));
    }
}
