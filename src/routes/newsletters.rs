use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{email_client::EmailClient, in_memory::AppState};

#[derive(Deserialize)]
pub struct NewsletterRequest {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    text: String,
    html: String,
}

pub async fn publish_newsletter(
    req: web::Json<NewsletterRequest>,
    email_client: web::Data<EmailClient>,
    app_state: web::Data<AppState>) -> HttpResponse {
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
        ).await.unwrap();
    }
    

    HttpResponse::Ok().finish()
}