use std::{borrow::Cow, mem::MaybeUninit, ptr, sync::LazyLock};
use memchr::memmem::{self, Finder};
use smallvec::SmallVec;
use url::{Host, Url};
use vc::{is_valid_email, is_valid_login, is_valid_phone_number};
use crate::{enums::{DataEnum, LineEnum, ValidationError}, schema::{LocalConfig, VULP}, ResultVULP};

static ANDROID_FIND: LazyLock<Finder<'static>> = LazyLock::new(|| memmem::Finder::new(b"==@"));
static HTTP_FIND: LazyLock<Finder<'static>> = LazyLock::new(|| memmem::Finder::new(b"http://"));
static HTTPS_FIND: LazyLock<Finder<'static>> = LazyLock::new(||memmem::Finder::new(b"https://"));

impl<'a> VULP<'a> {
    #[inline(always)]
    fn get_type(line: &[u8]) -> (LineEnum, SmallVec<[u8; 64]>) {
        let mut buffer = SmallVec::new();
        match () {
            _ if line.starts_with(b"http://") || line.starts_with(b"https://") => {
                buffer.extend_from_slice(line);
                (LineEnum::Http, buffer)
            }
            _ if line.starts_with(b"android://") => {
                buffer.extend_from_slice(line);
                (LineEnum::Android, buffer)
            }
            _ if ANDROID_FIND.find(line).is_some() => {
                buffer.reserve(10 + line.len());
                buffer.extend_from_slice(b"android://");
                buffer.extend_from_slice(line);
                (LineEnum::Android, buffer)
            }
            _ if HTTP_FIND.find(line).is_some() || HTTPS_FIND.find(line).is_some() => {
                buffer.extend_from_slice(line);
                (LineEnum::ReversedHttp, buffer)
            }
            _ => {
                buffer.extend_from_slice(line);
                (LineEnum::WithoutHttp, buffer)
            }
        }
    }

    #[inline(always)]
    fn validate_host(url: &str) -> Result<(Box<[u8]>, Option<u16>), ValidationError> {
        let parsed = Url::parse(url).map_err(|_| ValidationError::ParseError)?;
        let host = parsed.host().ok_or(ValidationError::ParseError)?;
        
        let (host_str, capacity) = match host {
            Host::Domain(d) => (d.to_string(), d.len() + 8),
            Host::Ipv4(ip) => (ip.to_string(), 22),
            Host::Ipv6(ip) => (ip.to_string(), 45),
        };

        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(b"https://");
        result.extend_from_slice(host_str.as_bytes());
        
        Ok((result.into_boxed_slice(), parsed.port()))
    }

    #[inline(always)]
    pub fn get_parts_in_line(&mut self, input: &'a [u8]) -> Result<(), ValidationError> {
        let mut process_line = SmallVec::<[u8; 128]>::new();
        process_line.resize(input.len(), 0);

        unsafe {
            let replace_chars = self.config.replace_chars.as_ptr();
            let replace_len = self.config.replace_chars.len();
            let src = input.as_ptr();
            let dst = process_line.as_mut_ptr();
            
            for i in 0..input.len() {
                let mut c = *src.add(i);
                for j in 0..replace_len {
                    if c == *replace_chars.add(j) {
                        c = b':';
                        break;
                    }
                }
                *dst.add(i) = c;
            }
        }

        let colon_count = process_line.iter().fold(0, |acc, &b| acc + (b == b':') as usize);
        if !(2..=5).contains(&colon_count) {
            return Err(ValidationError::ParseError);
        }

        let (linetype, line) = Self::get_type(&process_line);
        let mut parts = SmallVec::<[&[u8]; 6]>::new();
        let mut start = 0;
        unsafe {
            let bytes = line.as_ptr();
            let len = line.len();
            
            for i in 0..len {
                if *bytes.add(i) == b':' {
                    parts.push(std::slice::from_raw_parts(bytes.add(start), i - start));
                    start = i + 1;
                    if parts.len() == 5 { break }
                }
            }
            parts.push(std::slice::from_raw_parts(bytes.add(start), len - start));
        }

        let (login, password, url) = match linetype {
            LineEnum::ReversedHttp => {
                parts.reverse();
                (
                    parts.pop().map(|s| self::fast_filter(s, &self.config.bad_replace_chars)),
                    parts.pop().map(|s| s.into()),
                    Some(parts.join(&b':')),
                )
            },
            _ => (
                parts.pop().map(|s| self::fast_filter(s, &self.config.bad_replace_chars)),
                parts.pop().map(|s| s.into()),
                Some(parts.join(&b':')),
            ),
        };

        
        self.login = login.map(|l| {
            if self.config.login_to_lower_case {
                Cow::Owned(l.to_ascii_lowercase())
            } else {
                Cow::Owned(l)
            }
        });
        self.url = url.map(Cow::Owned);
        self.port = None;
        self.password = password.map(Cow::Owned);
        self.datatype = DataEnum::Unknown;
        self.linetype = match linetype {
            LineEnum::ReversedHttp => LineEnum::Http,
            _ => linetype,
        };

        if let Some(url) = &self.clone().url {
            if self.linetype == LineEnum::WithoutHttp {
                let mut new_url = Vec::with_capacity(url.len() + 8);
                new_url.extend_from_slice(b"https://");
                new_url.extend_from_slice(url);
                self.url = Some(Cow::Owned(new_url));
            }

            if self.linetype == LineEnum::Http {
                let (host, port) = Self::validate_host(std::str::from_utf8(&url).map_err(|_| ValidationError::ParseError)?).map_err(|_| ValidationError::ParseError)?;
                self.port = port;
                self.url = Some(Cow::Owned(host.to_vec()));
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn checking_bad_words_in_credits(&self) -> Result<(), ValidationError> {
        if self.config.filter_vector.is_empty() {
            return Ok(());
        }

        let login = self.login.as_deref().ok_or(ValidationError::FilterError)?;
        let password = self.password.as_deref().ok_or(ValidationError::FilterError)?;

        let mut combo = Vec::with_capacity(login.len() + password.len() + 1);
        combo.extend_from_slice(login);
        combo.push(b':');
        combo.extend_from_slice(password);

        let combo_lower = unsafe {
            let mut temp = combo.clone();
            ptr::write_bytes(temp.as_mut_ptr(), 0, temp.len());
            for (i, &c) in combo.iter().enumerate() {
                *temp.get_unchecked_mut(i) = c.to_ascii_lowercase();
            }
            temp
        };

        for pattern in &self.config.filter_vector {
            if !pattern.is_empty() && memmem::find(&combo_lower, pattern).is_some() {
                return Ok(());
            }
        }

        Err(ValidationError::FilterError)
    }

    #[inline(always)]
    fn find_type_credits(&mut self) -> Result<(), ValidationError> {
        if !self.config.find_data {
            self.datatype = DataEnum::Unknown;
            return Ok(());
        }

        let login = self.login.as_deref().ok_or(ValidationError::FindDataTypeError)?;

        let mut buffer = [MaybeUninit::<u8>::uninit(); 256];
        let len = login.len().min(255);
        
        unsafe {
            ptr::copy_nonoverlapping(login.as_ptr(), buffer.as_mut_ptr() as *mut u8, len);
            ptr::write(buffer[len].as_mut_ptr(), 0);
            
            let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                buffer.as_ptr() as *const u8,
                len
            ));

            if self.config.parse_email && is_valid_email(s.as_bytes()) {
                self.datatype = DataEnum::Email;
            } else if self.config.parse_login && is_valid_login(s.as_bytes()) {
                self.datatype = DataEnum::Login;
            } else if self.config.parse_number && is_valid_phone_number(s.as_bytes()) {
                self.datatype = DataEnum::Number;
            } else {
                return Err(ValidationError::FindDataTypeError);
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn validate_credit_length(&self) -> Result<(), ValidationError> {
        if !self.config.check_length {
            return Ok(());
        }

        let password_len = self.password.as_ref().map(|p| p.len()).unwrap_or(0);
        if !(self.config.password_length.0..=self.config.password_length.1).contains(&(password_len as u8)) {
            return Err(ValidationError::LengthError);
        }

        let login_len = self.login.as_ref().map(|l| l.len()).unwrap_or(0);
        let (min, max) = match self.datatype {
            DataEnum::Email => self.config.email_length,
            DataEnum::Login => self.config.login_length,
            DataEnum::Number => self.config.number_length,
            DataEnum::Unknown => self.config.default_length,
        };

        if !(min..=max).contains(&(login_len as u8)) {
            Err(ValidationError::LengthError)
        } else {
            Ok(())
        }
    }

    
    #[inline(always)]
    fn validate_full_length(&self) -> Result<(), ValidationError> {
        if !self.config.check_length {
            return Ok(())
        }
        match (self.config.length_full_line.0..=self.config.length_full_line.1).contains(&(self.full_line().len() as u8)) {
            true => Ok(()),
            false => Err(ValidationError::LengthError)
        }
    }

    #[inline(always)]
    fn check_equal(&self) -> Result<(), ValidationError> {
        if !self.config.check_equal {
            return Ok(());
        }

        match (&self.login, &self.password) {
            (Some(l), Some(p)) if l == p => Err(ValidationError::EqualError),
            _ => Ok(()),
        }
    }
    #[inline(always)]
    pub fn new(config: &LocalConfig<'a>) -> Self {
        Self {
            config: config.clone(),
            url: None,
            port: None,
            login: None,
            password: None,
            datatype: DataEnum::Unknown,
            linetype: LineEnum::WithoutHttp
        }
    }

    pub fn credits(&self) -> Vec<u8> {
        let login = self.login.as_ref().unwrap();
        let password = self.password.as_ref().unwrap();

        let mut result = Vec::with_capacity(login.len() + 1 + password.len());
        result.extend_from_slice(login);
        result.push(b':');
        result.extend_from_slice(password);

        result
    }

    pub fn full_line(&self) -> Vec<u8> {
        let url = self.url.as_ref().unwrap();
        let credit = self.credits();
        let mut result = Vec::with_capacity(url.len() + 5 + credit.len());
        result.extend_from_slice(url);
        result.push(b':');
        if let Some(port) = self.port {
            result.extend(&*port.to_string().as_bytes());
            result.push(b':');
        }
        result.extend_from_slice(&credit);
        result
    }

    #[inline(always)]
    pub fn validate(&mut self, line: &'a [u8]) -> Result<ResultVULP, ValidationError> {
        self.login = None;
        self.password = None;
        self.url = None;
        self.port = None;
        self.datatype = DataEnum::Unknown;
        self.linetype = LineEnum::WithoutHttp;
        self.get_parts_in_line(line)?;
        self.checking_bad_words_in_credits()?;
        self.find_type_credits()?;
        self.check_equal()?;
        self.validate_full_length()?;
        self.validate_credit_length()?;
        Ok(ResultVULP::from(self.clone()))
    }
}

#[inline(always)]
fn fast_filter(s: &[u8], bad_chars: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len());
    let bad_ptr = bad_chars.as_ptr();
    let bad_len = bad_chars.len();
    
    unsafe {
        let src = s.as_ptr();
        for i in 0..s.len() {
            let mut c = *src.add(i);
            for j in 0..bad_len {
                if c == *bad_ptr.add(j) {
                    c = 0;
                    break;
                }
            }
            if c != 0 {
                result.push(c);
            }
        }
    }
    result.shrink_to_fit();
    result
}