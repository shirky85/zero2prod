use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{email_client::EmailClient, in_memory::{AppState, Subscription}};

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



impl SubscriptionRequest {
    pub fn new(username: String, email: String) -> Self {
        SubscriptionRequest { username, email }
    }
}


#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(info, app_state, email_client),
    fields(
    %info.email,
    %info.username
    )
)]
pub async fn subscribe(
    info: web::Json<SubscriptionRequest>, 
    app_state: web::Data<AppState>,
    email_client: web::Data<EmailClient>,
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
    };

    subscriptions.push(subscription);

    if email_client.send_email(
        info.email.clone(),
        "welcome",
        "Welcome to our newsletter!",
        "Welcome to our newsletter!",
    )
    .await.is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(id)
}

