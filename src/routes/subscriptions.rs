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



pub async fn subscribe(info: web::Json<SubscriptionRequest>, app_state: web::Data<AppState>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    if info.username == ""{
        log::warn!("request_id {} - request for email {} was missing username", request_id, info.email);
        return HttpResponse::BadRequest().body("username is missing")
    }
    if info.email == ""{
        log::warn!("request_id {} - request for username {} was missing email", request_id, info.username);
        return HttpResponse::BadRequest().body("email is missing")
    }
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        info.email,
        info.username
        );

    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let id = app_state.get_id();
    let subscription = Subscription{
        id: id,
        username: info.username.to_string(),
        email: info.email.to_string(),
    };



    subscriptions.push(subscription);
    log::info!(
        "request_id {} - Added '{}' '{}' as a new subscriber successfully with id {}.",
        request_id,
        info.email,
        info.username,
        id
        );

    HttpResponse::Ok().json(id)
    //HttpResponse::Ok().finish()
}

