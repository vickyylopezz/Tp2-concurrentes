use crate::payment_method::Method;

#[derive(Debug, PartialEq)]
pub enum Action {
    Block(u32),
    CompleteOrder(u32, u32, Method),
    Ack,
    NotEnoughPoints(u32),
    ClientAlreadyBlocked(u32),
    FailOrder(u32),
}
