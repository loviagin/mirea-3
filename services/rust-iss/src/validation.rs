use serde_json::Value;
use crate::domain::ApiError;

pub struct Validator;

impl Validator {
    pub fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
        let limit = limit.unwrap_or(20);
        if limit < 1 || limit > 1000 {
            return Err(ApiError::Validation(
                "limit must be between 1 and 1000".to_string()
            ));
        }
        Ok(limit)
    }
    
    pub fn validate_source(source: &str) -> Result<(), ApiError> {
        let allowed = ["apod", "neo", "flr", "cme", "spacex"];
        if !allowed.contains(&source.to_lowercase().as_str()) {
            return Err(ApiError::Validation(
                format!("source must be one of: {}", allowed.join(", "))
            ));
        }
        Ok(())
    }
    
    pub fn validate_json_payload(payload: &Value) -> Result<(), ApiError> {
        if !payload.is_object() && !payload.is_array() {
            return Err(ApiError::Validation(
                "payload must be a JSON object or array".to_string()
            ));
        }
        Ok(())
    }
}

