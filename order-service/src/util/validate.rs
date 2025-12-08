use crate::core::error::AppError;
use axum::extract::rejection::JsonRejection;
use axum::Json;
use validator::{Validate, ValidationError};
pub fn handle_json_rejection<T: Validate>(
    req: &Result<Json<T>, JsonRejection>,
) -> Result<(), AppError> {
    match req {
        Ok(value) => {
            if let Err(e) = value.validate() {
                Err(AppError::BadRequestError(e.to_string()))
            } else {
                Ok(())
            }
        },
        Err(JsonRejection::JsonDataError(err)) => Err(AppError::BadRequestError(err.body_text())),
        Err(JsonRejection::JsonSyntaxError(err)) => Err(AppError::BadRequestError(err.body_text())),
        Err(JsonRejection::BytesRejection(err)) => Err(AppError::BadRequestError(err.body_text())),
        Err(_) => Err(AppError::BadRequestError("Unknown error".to_string())),
    }
}

pub fn validate_special_characters(input: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^[a-zA-ZÀ-ỹ0-9 _()\-\?,!]+$").unwrap();
    if re.is_match(input) {
        Ok(())
    } else {
        Err(ValidationError::new("terrible_username"))
    }
}
