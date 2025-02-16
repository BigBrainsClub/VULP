mod enums;
mod impls;
mod schema;
mod validator;

pub use schema::{VULP, LocalConfig};
pub use enums::*;


#[cfg(test)]
mod test_android_lines {
    use crate::{VULP, LocalConfig};
    #[test]
    fn test_valid_lines() {
        let mut new_validator = VULP::new(&LocalConfig::default());
        let lines: Vec<&[u8]> = vec![
            b"android://ojqioehtwpefnweuetwerwe@==example.com:gnweugbwekfnwe:ifaentqiewwer",
            b"android //aidngqeqeoingqpeiorqeut@==testing.com ianitgnweutnwer tqintuweujbg",
            b"android;//gisdgweiuhtweihtierhteirh@==testing.com;09u5239509237523;9235923849234",
            b"android://gjweitjwirhygwiehtriwehr@==testing.com;owjeotweorowpeutwer wietiwehtiwer",
            b"pdgjiwopethweirwejfiewf@==testing.com;gwoejhtiwehriete:ihreigtertwer",
            b"https://example.com:examplelogin:giwegwegwegew",
        ];
        for line in lines {
            assert!( match new_validator.validate(line) {
                Ok(_) => true,
                e => {print!("{:?}, {}", e, String::from_utf8(line.to_vec()).unwrap()); false}
            } )
        }
    }
}