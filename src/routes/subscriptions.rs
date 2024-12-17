use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::in_memory::{AppState, Subscription};


#[derive(Deserialize,Serialize)]
pub struct SubscriptionRequest {
    username: String,
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

