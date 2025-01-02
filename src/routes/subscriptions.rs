use actix_web::{web, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{email_client::EmailClient, in_memory::{AppState, Subscription}, startup::ApplicationBaseUrl};

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\sa-zA-Z0-9_]+$").unwrap()
});
// TODO: should we export this to another file?
#[derive(thiserror::Error)]
pub enum SubscriptionError {
    #[error("Validation error(s): {0}")]
    ValidationError(String),
    #[error("Subscription already exists: {0}")]
    AlreadyExists(serde_json::Value),
    #[error("Failed to send email")]
    SendEmailError(#[from]reqwest::Error),
}

// TODO: Ask AI to explain this syntax later or simplify it
fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}


impl std::fmt::Debug for SubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscriptionError {
    fn error_response(&self) -> HttpResponse {
        match self {
            SubscriptionError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(json!({ "message": errors }))
            }
            SubscriptionError::AlreadyExists(message) => {
                HttpResponse::BadRequest().json(message)
            }
            SubscriptionError::SendEmailError(_err) => {
                HttpResponse::BadGateway().finish()
            }
        }
    }
}


#[derive(Deserialize,Serialize,Validate)]
pub struct SubscriptionRequest {
    #[validate(length(min = 2, max = 100))]
    #[validate(regex(path = *NAME_REGEX))]
    username: String,
    #[validate(email)]
    email: String,
}

async fn send_confirmation_email(
    email_client: &EmailClient, 
    recepient: String,
    subscription_id: i32,
    base_url: &String) 
    -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, subscription_id);
    email_client
        .send_email(
            recepient,
            "Welcome!",
            &format!("Welcome to our newsletter!<br />\
                            Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link),
            &format!("Welcome to our newsletter!\nPlease confirm your subscription by clicking on the link: {}", confirmation_link),
        )
        .await
}



impl SubscriptionRequest {
    pub fn new(username: String, email: String) -> Self {
        SubscriptionRequest { username, email }
    }
}


#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(info, app_state, email_client, base_url),
    fields(
    %info.email,
    %info.username
    )
)]
pub async fn subscribe(
    info: web::Json<SubscriptionRequest>, 
    app_state: web::Data<AppState>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscriptionError> {
    match info.validate() {
        Ok(_) => println!("Request for subscribe passed validation"),
        Err(errors) => {return Err(SubscriptionError::ValidationError(errors.to_string()));}
    }
    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let email = &info.email;
    // Find the subscription with the matching token
    let mut new_id: Option<i32> = None;
    if let Some(subscription) = subscriptions.iter_mut().find(|s| s.email == *email) {
        // Subscription already exists
        match subscription.status.as_str() {
            "confirmed" => {return Err(SubscriptionError::AlreadyExists(serde_json::json!({ "message": format!("Subscription with email {} is already confirmed", email) })));},
            _ =>  {new_id = Some(subscription.id)}, // If email not confirmed resend the confimation link
        }
    } else {
        let id = app_state.get_id();
        new_id = Some(id);
        let subscription = Subscription{
            id: id,
            username: info.username.to_string(),
            email: info.email.to_string(),
            status: "pending_confirmation".to_string(),
        };
    
        subscriptions.push(subscription);
    }

    let _response = send_confirmation_email(&email_client, info.email.clone(), new_id.unwrap(), &base_url.0)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": new_id.unwrap() })))
}

#[derive(serde::Deserialize)]
pub struct SubscriptionParameters {
    subscription_id: String
}

pub async fn get_subscription(
    app_state: web::Data<AppState>,
    id: web::Query<SubscriptionParameters>
) -> HttpResponse {
    let subscriptions = app_state.subscriptions.read().expect("RwLock poisoned");
    if let Some(subscription) = subscriptions.iter().find(|&s| s.id.to_string() == id.subscription_id) {
        return HttpResponse::Ok().json(subscription);
    }
    return HttpResponse::NotFound().finish();
}

