#[derive(Debug, PartialEq)]
pub enum Error {
    NotFileInput,
    FileNotFound,
    WrongFileFormat,
    NoMoreOrders,
    CantSendMessage,
    CantCloneSocket,
    Timeout,

}
