use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::server::ServerError;
use regex::Regex;

pub const MIN_LEN_PASSOWRD: usize = 6;
pub const SESSION_KEY: &str = "ssid";

pub fn hash_password(password: &str) -> Result<String, ServerError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ServerError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| ServerError::InternalServerError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn is_valid_password(password: &str) -> bool {
    password.len() >= MIN_LEN_PASSOWRD
}

pub fn is_valid_email(email: &str) -> Result<bool, ServerError> {
    let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?;

    Ok(regex.is_match(email))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_emails() {
        let emails = [
            "simple@example.com",
            "very.common@example.com",
            "disposable.style.email.with+symbol@example.com",
            "other.email-with-hyphen@example.com",
            "fully-qualified-domain@example.co.uk",
            "user.name+tag+sorting@example.com",
            "x@example.com",
            "example-indeed@strange-example.com",
            "user%example.com@example.org",
            "user_name@example.org",
            "user123@sub.domain.example.com",
            "example@s.solution",
            "a@b.co",
        ];

        for email in emails.iter() {
            assert!(
                is_valid_email(email).unwrap(),
                "failed for email: {:?}",
                email
            );
        }
    }

    #[test]
    fn invalid_emails() {
        let emails = [
            "admin@mailserver1",
            "\"john..doe\"@example.org",
            "\"much.more unusual\"@example.com",
            "\"very.unusual.@.unusual.com\"@example.com",
            "\"very.(),:;<>[]\".VERY.\"very@\\ \"very\".unusual\"@strange.example.com",
            "test@xn--d1acufc.xn--p1ai",
            "test@пример.рф",
        ];

        for email in emails.iter() {
            assert!(
                !is_valid_email(email).unwrap(),
                "failed for email: {:?}",
                email
            );
        }
    }

    #[test]
    fn valid_password() {
        let password = "anaksfdb3434bbc";
        assert!(is_valid_password(password));
    }

    #[test]
    fn invalid_password() {
        let password = "anak3";
        assert!(!is_valid_password(password));
    }
}
