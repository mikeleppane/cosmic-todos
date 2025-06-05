use validator::ValidationError;

/// Validates that the input string does not contain HTML tags.
///
/// # Errors
///
/// Returns a `ValidationError` with code "`contains_html`" if the input contains HTML tags.
pub fn validate_no_html(input: &str) -> Result<(), ValidationError> {
    let Ok(html_tags) = regex::Regex::new(r"<[^>]*>") else {
        return Err(ValidationError::new("regex_compilation_error"));
    };
    if html_tags.is_match(input) {
        return Err(ValidationError::new("contains_html"));
    }
    Ok(())
}

/// Validates that the timestamp represents a future date.
///
/// # Errors
///
/// Returns a `ValidationError` with code "`must_be_future_date`" if the timestamp
/// is not in the future relative to the current UTC time.
pub fn validate_future_date(timestamp: u64) -> Result<(), ValidationError> {
    let now = chrono::Utc::now().timestamp();
    let timestamp_i64 = timestamp.try_into().unwrap_or(i64::MAX);
    if now < 0 || timestamp_i64 <= now {
        return Err(ValidationError::new("must_be_future_date"));
    }
    Ok(())
}
