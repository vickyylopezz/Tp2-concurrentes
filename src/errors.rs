#[derive(Debug, PartialEq)]
pub enum Error {
    NotFileInput,
    FileNotFound,
    WrongFileFormat,
}
