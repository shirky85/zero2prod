use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};


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



pub async fn subscribe(info: web::Json<SubscriptionRequest>,) -> HttpResponse {
    println!("{}",info.username);
    println!("{}",info.email);
    if info.username == ""{
        return HttpResponse::BadRequest().body("username is missing")
    }
    if info.email == ""{
        return HttpResponse::BadRequest().body("email is missing")
    }

    HttpResponse::Ok().finish()
}