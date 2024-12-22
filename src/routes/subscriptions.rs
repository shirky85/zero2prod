use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::in_memory::{AppState, Subscription};
use once_cell::sync::Lazy;
use regex::Regex;

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
    skip(info, app_state),
    fields(
    %info.email,
    %info.username
    )
)]
pub async fn subscribe(info: web::Json<SubscriptionRequest>, app_state: web::Data<AppState>) -> HttpResponse {
    match info.validate() {
        Ok(_) => println!("Request for subscribe passed validation"),
        Err(errors) => {return HttpResponse::BadRequest().body(errors.to_string());}
    }
    let request_id = Uuid::new_v4();
    if info.username == ""{
        tracing::warn!("request_id {} - request for email {} was missing username", request_id, info.email);
        return HttpResponse::BadRequest().body("username is missing")
    }
    if info.email == ""{
        tracing::warn!("request_id {} - request for username {} was missing email", request_id, info.username);
        return HttpResponse::BadRequest().body("email is missing")
    }

    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let id = app_state.get_id();
    let subscription = Subscription{
        id: id,
        username: info.username.to_string(),
        email: info.email.to_string(),
    };

    subscriptions.push(subscription);

    HttpResponse::Ok().json(id)
}

