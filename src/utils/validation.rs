use crate::domain::auth::UserInfo;

/// Validates user credentials and returns user information if valid.
///
/// # Errors
///
/// Returns an error string if the provided username and password combination
/// does not match any known valid credentials.
pub fn validate_credentials(username: &str, password: &str) -> Result<UserInfo, String> {
    // Simple hardcoded validation for demo
    // In production, you'd check against a database or external service
    match (username, password) {
        ("Mikko", "password123") => Ok(UserInfo {
            username: "Mikko".to_string(),
            email: "mikko@familyleppanen.com".to_string(),
            display_name: "Mikko Leppänen".to_string(),
        }),
        ("Niina", "password123") => Ok(UserInfo {
            username: "Niina".to_string(),
            email: "niina@familyleppanen.com".to_string(),
            display_name: "Niina Leppänen".to_string(),
        }),
        _ => Err("Invalid credentials".to_string()),
    }
}
