#![allow(dead_code, unused_variables)]

pub async fn generate_bom_entry(new_order_idx: i64) {
    tracing::info!("Generating BOM entry for new order {}", new_order_idx);

    let order = db_api::get_order(new_order_idx).await;
}
