use crate::payment_method::Method;

#[derive(Debug, PartialEq)]
pub enum Action {
    Block(u32),
    CompleteOrder(u32, u32, Method),
    FailOrder(u32),
    NotEnoughPoints(u32),
    Ack,
}
