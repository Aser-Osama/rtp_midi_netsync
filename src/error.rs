pub enum Error {
    ParseError(ParseError),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Insufficient,
    InvalidHeader,
    InvalidData,
}
