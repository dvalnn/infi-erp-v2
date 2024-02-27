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
