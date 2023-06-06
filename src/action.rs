use crate::method::Method;

pub enum Action {
    Block(u32),
    CompleteOrder(u32, u32, Method),
    FailOrder(u32),
}
