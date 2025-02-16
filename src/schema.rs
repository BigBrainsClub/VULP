use std::{borrow::Cow, ops::Deref};
use smallvec::SmallVec;
use crate::enums::{DataEnum, LineEnum};

#[derive(Clone, Debug)]
pub struct VULP<'a> {
    pub config: LocalConfig<'a>,
    pub url: Option<Cow<'a, [u8]>>,
    pub port: Option<u16>,
    pub login: Option<Cow<'a, [u8]>>,
    pub password: Option<Cow<'a, [u8]>>,
    pub datatype: DataEnum,
    pub linetype: LineEnum,
}

impl<'a, T> AsRef<T> for VULP<'a> 
where
    VULP<'a>: Deref<Target = T>,
{
    fn as_ref(&self) -> &T {
        self
    }
}


#[derive(Clone, Debug)]
pub struct LocalConfig<'a> {
    pub replace_chars: Cow<'a, [u8]>,
    pub bad_replace_chars: Cow<'a, [u8]>,
    pub login_to_lower_case: bool,
    pub filter_vector: SmallVec<[SmallVec<[u8; 16]>; 4]>,
    pub parse_email: bool,
    pub parse_login: bool,
    pub parse_number: bool,
    pub find_data: bool,
    pub check_equal: bool,
    pub check_length: bool,
    pub length_full_line: (u8, u8),
    pub email_length: (u8, u8),
    pub login_length: (u8, u8),
    pub number_length: (u8, u8),
    pub default_length: (u8, u8),
    pub password_length: (u8, u8),
}

impl<'a> LocalConfig<'a> {
    pub fn new(
        replace_chars: impl Into<Cow<'a, [u8]>>,
        bad_replace_chars: impl Into<Cow<'a, [u8]>>,
        filter_vector: impl Into<SmallVec<[SmallVec<[u8; 16]>; 4]>>,
        length_full_line: (u8, u8),
        email_length: (u8, u8),
        login_length: (u8, u8),
        number_length: (u8, u8),
        default_length: (u8, u8),
        password_length: (u8, u8),
        login_to_lower_case: bool,
        parse_email: bool,
        parse_login: bool,
        parse_number: bool,
        find_data: bool,
        check_equal: bool,
        check_length: bool,
    ) -> Self {
        Self {
            replace_chars: replace_chars.into(),
            bad_replace_chars: bad_replace_chars.into(),
            filter_vector: filter_vector.into(),
            login_to_lower_case,
            parse_email,
            parse_login,
            parse_number,
            find_data,
            check_equal,
            check_length,
            length_full_line,
            email_length,
            login_length,
            number_length,
            default_length,
            password_length,
        }
    }
}

impl<'a> Default for LocalConfig<'a> {
    fn default() -> Self {
        Self {
            replace_chars: Cow::Borrowed(b";| "),
            bad_replace_chars: Cow::Borrowed(b"()*$!%&^#<>?~=[]+/\\,"),
            login_to_lower_case: true,
            filter_vector: SmallVec::new(),
            parse_email: true,
            parse_login: true,
            parse_number: true,
            find_data: true,
            check_equal: true,
            check_length: true,
            length_full_line: (20, 150),
            email_length: (8, 35),
            login_length: (5, 35),
            number_length: (11, 16),
            default_length: (5, 35),
            password_length: (8, 35),
        }
    }
}

#[derive(Debug)]
pub struct ResultVULP {
    pub full_line: Vec<u8>,
    pub credits: Vec<u8>,
    pub datatype: DataEnum,
    pub linetype: LineEnum,
}

impl <'a> From<VULP<'a>> for ResultVULP {
    fn from(value: VULP) -> ResultVULP {
        Self {
            credits: value.credits(),
            full_line: value.full_line().to_owned(),
            datatype: value.datatype,
            linetype: value.linetype
        }
    }
}