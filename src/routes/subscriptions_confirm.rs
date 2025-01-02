use actix_web::{web, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::in_memory::AppState;

#[derive(thiserror::Error, Debug)]
pub enum ConfirmError {
    #[error("Validation error(s): {0}")]
    ValidationError(String),
    #[error("Email was not confirmed: {0}")]
    NotFound(serde_json::Value),
}
impl ResponseError for ConfirmError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ConfirmError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(json!({ "message": errors }))
            }
            ConfirmError::NotFound(message) => {
                HttpResponse::NotFound().json(message)
            }
        }
    }
}

#[derive(Deserialize,Serialize,Debug,Validate)]
pub struct Parameters {
    #[validate(length(min = 1, max = 8))]
    subscription_token: String
}

#[tracing::instrument(
    name = "confirming a new subscriber",
    skip(app_state),
    fields(%parameters.subscription_token)
)]
pub async fn subscription_confirm(app_state: web::Data<AppState>,
    parameters: web::Query<Parameters>
) -> Result<HttpResponse, ConfirmError> {
    match parameters.validate() {
        Ok(_) => println!("Request for confirm passed validation"),
        Err(errors) => {return Err(ConfirmError::ValidationError(errors.to_string()));}
    }
    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let num_token = parameters.subscription_token.parse::<i32>().unwrap();
    // Find the subscription with the matching token
    if let Some(subscription) = subscriptions.iter_mut().find(|s| s.id == num_token) {
        // Update the status to "confirmed"
        subscription.status = "confirmed".to_string();
        Ok(HttpResponse::Ok().json(json!({
            "message": "Subscription confirmed successfully"
        })))
    } else {
        Err(ConfirmError::NotFound(json!({
            "error": "Subscription not found"
        })))
    }
}