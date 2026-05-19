pub mod jwt;
pub mod hash;
pub mod email;

// normalize email — lowercase and trim
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}