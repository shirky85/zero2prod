use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{email_client::EmailClient, in_memory::{AppState, Subscription}, startup::ApplicationBaseUrl};

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\sa-zA-Z0-9_]+$").unwrap()
});


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
) -> HttpResponse {
    match info.validate() {
        Ok(_) => println!("Request for subscribe passed validation"),
        Err(errors) => {return HttpResponse::BadRequest().body(errors.to_string());}
    }
    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let id = app_state.get_id();
    let subscription = Subscription{
        id: id,
        username: info.username.to_string(),
        email: info.email.to_string(),
        status: "pending_confirmation".to_string(),
    };

    subscriptions.push(subscription);

    if send_confirmation_email(&email_client, info.email.clone(), id, &base_url.0)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(id)
}

#[derive(serde::Deserialize)]
pub struct SubscriptionParameters {
    subscription_id: String
}

pub async fn get_subscription(
    app_state: web::Data<AppState>,
    id: web::Query<SubscriptionParameters>
) -> HttpResponse {
    HttpResponse::Ok().json(Subscription{
        id: 2,
        username: "something".to_string(),
        email: "something@gmail.com".to_string(),
        status: "registered".to_string(),
    })
}

