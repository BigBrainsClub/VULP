use crate::enums::{DataEnum, LineEnum};

impl std::fmt::Display for DataEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataEnum::Email => "email",
                DataEnum::Login => "login",
                DataEnum::Number => "number",
                DataEnum::Unknown => "unknown"
            }
        )
    }
}


impl std::fmt::Display for LineEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LineEnum::Android => "android",
                LineEnum::Http => "http",
                LineEnum::ReversedHttp => "reversed_http",
                LineEnum::WithoutHttp => "without_http"
            }
        )
    }
}