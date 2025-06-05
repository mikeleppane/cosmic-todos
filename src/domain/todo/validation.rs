use validator::ValidationError;

pub fn validate_no_html(input: &str) -> Result<(), ValidationError> {
    let html_tags = regex::Regex::new(r"<[^>]*>").unwrap();
    if html_tags.is_match(input) {
        return Err(ValidationError::new("contains_html"));
    }
    Ok(())
}

pub fn validate_future_date(timestamp: u64) -> Result<(), ValidationError> {
    let now = chrono::Utc::now().timestamp() as u64;
    if timestamp <= now {
        return Err(ValidationError::new("must_be_future_date"));
    }
    Ok(())
}
