use regex::regex;
use serde::{Deserialize};
use validator::{Validate, ValidationError};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct SignUpRequest {
    #[validate(length(min = 1, max = 16))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.is_ascii() {
        return Err(ValidationError::new("Password must only have ascii characters"));
    }
    if password.len() < 8 {
        return Err(ValidationError::new("Password is too short"));
    }
    if password.len() > 16 {
        return Err(ValidationError::new("Password is too long"));
    }
    if password.to_ascii_lowercase().eq(password) {
        return Err(ValidationError::new("Password must have at least one uppercase letter"));
    }
    if password.to_ascii_uppercase().eq(password) {
        return Err(ValidationError::new("Password must have at least one lowercase letter"));
    }
    if !regex!(".*[0-9].*").is_match(password) {
        return Err(ValidationError::new("Password must have at least one digit"));
    }
    if !regex!(".*[$&+,:;=?@#|'<>.^*()%!-].*").is_match(password) {
        return Err(ValidationError::new("Password must have at least one special character"));
    }
    Ok(())
}