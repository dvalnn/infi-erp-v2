use sqlx::{postgres::types::PgMoney, PgPool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tools {
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    INVALID,
}

impl From<String> for Tools {
    fn from(value: String) -> Self {
        match value.as_str() {
            "T1" => Tools::T1,
            "T2" => Tools::T2,
            "T3" => Tools::T3,
            "T4" => Tools::T4,
            "T5" => Tools::T5,
            "T6" => Tools::T6,
            _ => Tools::INVALID,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transformation {
    pub id: i64,
    pub from_piece: i64,
    pub to_piece: i64,
    pub tool: Tools,
    pub quantity: i32,
    pub cost: PgMoney,
}

pub type Recipe = Vec<Transformation>;

/// Gets transformations that output "to_piece".
pub async fn get_imediate_recipe(
    to_piece: i64,
    pool: &PgPool,
) -> Result<Recipe, sqlx::Error> {
    sqlx::query_as!(
        Transformation,
        "SELECT * FROM transformations WHERE to_piece = $1",
        to_piece
    )
    .fetch_all(pool)
    .await
}

/// Gets the full recipe required to produce the piece.
/// Traverses the transformations table until it finds the root piece.
/// Return a flat list of transformations that represents the recipe tree.
pub async fn get_repice_to_root(
    final_piece_id: i64,
    pool: &PgPool,
) -> Result<Recipe, sqlx::Error> {
    let mut targets = vec![final_piece_id];
    let mut recipe: Recipe = Vec::new();

    loop {
        let mut transforms = Vec::new();

        for piece in targets.iter() {
            let mut tfs = get_imediate_recipe(*piece, pool).await?;
            transforms.append(&mut tfs);
        }

        if transforms.is_empty() {
            break;
        }

        transforms.dedup();
        targets.clear();
        for t in transforms.iter() {
            targets.push(t.from_piece);
        }

        recipe.append(&mut transforms);
    }

    Ok(recipe)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bom {
    pub id: i64,
    pub order_id: i64,
    pub transformation_id: i64,
    pub piece_number: i32,
    pub pieces_total: i32,
    pub step_number: i32,
    pub steps_total: i32,
}

impl Bom {
    pub fn new(
        order_id: i64,
        transformation_id: i64,
        piece_number: i32,
        pieces_total: i32,
        step_number: i32,
        steps_total: i32,
    ) -> Bom {
        Bom {
            id: 0, // id is auto-generated by the database upon insertion
            order_id,
            transformation_id,
            piece_number,
            pieces_total,
            step_number,
            steps_total,
        }
    }

    pub async fn get_by_id(id: i64, pool: &PgPool) -> Result<Bom, sqlx::Error> {
        sqlx::query_as!(Bom, "SELECT * FROM bom WHERE id = $1", id)
            .fetch_one(pool)
            .await
    }

    /// Inserts a batch of BOM entries into the database.
    /// The entries are inserted in a single transaction.
    /// If any of the entries fail to be inserted, the transaction
    /// is rolled back and the error is returned.
    /// If all entries are inserted successfully, a notification is
    /// send to the `NewBomEntry` channel with the ids of the inserted
    /// entries separated by commas.
    pub async fn insert_batch(
        batch: &[Bom],
        pool: &PgPool,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let mut ids = Vec::new();

        for entry in batch {
            let Bom {
                id: _, // id is auto-generated by the database upon insertion
                order_id,
                transformation_id,
                piece_number,
                pieces_total,
                step_number,
                steps_total,
            } = entry;
            let id = sqlx::query!(
                "
            INSERT INTO bom(
                order_id,
                transformation_id,
                piece_number,
                pieces_total,
                step_number,
                steps_total
            )
            VALUES($1, $2, $3, $4, $5, $6)
            RETURNING id
            ",
                order_id,
                transformation_id,
                piece_number,
                pieces_total,
                step_number,
                steps_total
            )
            .fetch_one(&mut *tx)
            .await?
            .id;

            ids.push(id);
        }

        let channel = crate::NotificationChannel::NewBomEntry;
        let ids = ids
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!("NOTIFY {}, '{}'", channel, ids);
        sqlx::query(&query).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }
}
