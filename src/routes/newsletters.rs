use actix_web::{web, HttpResponse, ResponseError};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::{email_client::EmailClient, in_memory::AppState};

use super::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Validation error(s): {0}")]
    ValidationError(String),
    #[error("Failed to send newsletter email")]
    SendEmailError(#[from]reqwest::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(json!({ "message": errors }))
            }
            PublishError::SendEmailError(_err) => {
                HttpResponse::BadGateway().finish()
            }
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct NewsletterRequest {
    #[validate(length(min = 5, max = 50))]
    title: String,
    #[validate()]
    content: Content,
}

#[derive(Deserialize, Validate)]
struct Content {
    #[validate(length(min = 10, max = 500))]
    text: String,
    #[validate(length(min = 10, max = 500))]
    html: String,
}

#[tracing::instrument(
    name = "Publishing a newsletter",
    skip(req, email_client, app_state),
    fields(
        %req.title,
    )
)]
pub async fn publish_newsletter(
    req: web::Json<NewsletterRequest>,
    email_client: web::Data<EmailClient>,
    app_state: web::Data<AppState>) -> Result<HttpResponse, PublishError> {
    match req.validate() {
        Ok(_) => match req.content.validate(){
            Ok(_) => println!("Request for publish passed validation"),
            Err(errors) => {return Err(PublishError::ValidationError(errors.to_string()));}
        },
        Err(errors) => {return Err(PublishError::ValidationError(errors.to_string()));}
    }

    let subscriptions = app_state.subscriptions.read().expect("RwLock poisoned");
    let recipients: Vec<String> = subscriptions.iter()
        .filter(|s| s.status == "confirmed")
        .map(|s| s.email.clone()).collect();
    if !recipients.is_empty(){
        email_client.send_email(
            recipients,
            &req.title,
            &req.content.html,
            &req.content.text
        ).await?;
    }
    

    Ok(HttpResponse::Ok().finish())
}