use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, postgres::types::PgMoney, PgPool};

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
pub struct PgOrder {
    pub id: i64,
    pub piece_id: i64,
    pub client_id: i64,
    pub number: i32,
    pub quantity: i32,
    pub due_date: i32,
    pub late_pen: PgMoney,
    pub early_pen: PgMoney,
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
    let piece_id = tx_get_piece_id(&order.work_piece, &mut tx).await?;
    let client_id = match tx_get_client_id(client, &mut tx).await {
        Ok(id) => {
            tracing::debug!("Client found! ID: {}", id);
            id
        }
        Err(_) => {
            tracing::debug!("Client not found! Creating new client");
            sqlx::query!(
                "INSERT INTO clients(name) VALUES($1) RETURNING id",
                client.name_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        }
    };

    let order = PgOrder {
        id: 0, // This will be set by the database when the order is inserted
        piece_id,
        client_id,
        number: order.number,
        quantity: order.quantity,
        due_date: order.due_date,
        late_pen: PgMoney(late_penalty),
        early_pen: PgMoney(early_penalty),
    };

    tracing::debug!("Placing order: {:#?}", order);
    let order_id = match place_new_order(order, &mut tx).await {
        Ok(id) => id,
        Err(e) => return Err(e.into()),
    };

    let channel = crate::NotificationChannel::NewOrder;
    let query = format!("NOTIFY {}, '{}'", channel, order_id);
    sqlx::query(&query).execute(&mut *tx).await?;
    tx.commit().await?;

    Ok(order_id)
}

async fn place_new_order(
    order: PgOrder,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<i64> {
    Ok(sqlx::query!(
        "INSERT INTO orders (
            piece_id,
            client_id,
            number,
            quantity,
            due_date,
            late_pen,
            early_pen
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
        order.late_pen,
        order.early_pen
    )
    .fetch_one(&mut **tx)
    .await?
    .id)
}

/// Get the id of a client. Use with a transaction type connection
pub async fn tx_get_client_id(
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

/// Get the id of a piece. Use with a transaction type connection
pub async fn tx_get_piece_id(
    piece_name: &WorkPieces,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<i64> {
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

pub async fn run_migrations(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn get_order(
    new_order_id: i64,
    pool: &PgPool,
) -> sqlx::Result<PgOrder> {
    sqlx::query_as!(PgOrder, "SELECT * FROM orders WHERE id = $1", new_order_id)
        .fetch_one(pool)
        .await
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
