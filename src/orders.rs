use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub payment_method: String,
}
