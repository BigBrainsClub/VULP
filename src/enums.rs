#[derive(Clone, Debug)]
pub enum DataEnum {
    Login,
    Email,
    Number,
    Unknown
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineEnum {
    Http,
    Android,
    ReversedHttp,
    WithoutHttp,
    Unknown
}

#[derive(Debug)]
pub enum ValidationError {
    LengthError,
    FindDataTypeError,
    ParseError,
    FilterError,
    EqualError
}