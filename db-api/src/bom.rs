use sqlx::{postgres::types::PgMoney, PgPool};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct Transformation {
    pub id: i64,
    pub from_piece: i64,
    pub to_piece: i64,
    pub tool: Tools,
    pub quantity: i32,
    pub cost: PgMoney,
}

pub struct Recipe(pub Vec<Transformation>);

pub async fn get_piece_recipe(
    piece_id: i64,
    pool: &PgPool,
) -> Result<Vec<Recipe>, sqlx::Error> {
    let mut from_piece = piece_id;
    let mut recipes: Vec<Recipe> = Vec::new();

    loop {
        todo!("Get the recipe from the database");
    }

    Ok(recipes)
}
