use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

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
    println!("{}",info.username);
    println!("{}",info.email);
    if info.username == ""{
        return HttpResponse::BadRequest().body("username is missing")
    }
    if info.email == ""{
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
    //HttpResponse::Ok().finish()
}

