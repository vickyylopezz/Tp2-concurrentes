use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub customer_id: u32,
    pub payment_method: String,
}
