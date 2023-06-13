use crate::payment_method::Method;
#[derive(PartialEq)]
pub enum Action {
    Block(u32, u32),
    CompleteOrder(u32, u32, Method, u32),
    FailOrder(u32, u32),
    ClientAlreadyBlocked(u32),
    NotEnoughPoints(u32),
    Update(u32, i32, bool),
    Ack,
    Try,
    Up,
    Down,
    Sync(u32),
    SyncStart,
    SyncPart(String),
    SyncEnd,
}
