use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::in_memory::AppState;

#[derive(Deserialize,Serialize,Debug)]
pub struct Parameters {
    subscription_token: String
}

#[tracing::instrument(
    name = "confirming a new subscriber",
    skip(app_state),
    fields(%parameters.subscription_token)
)]
pub async fn subscription_confirm(app_state: web::Data<AppState>,
    parameters: web::Query<Parameters>
) -> HttpResponse {
    let mut subscriptions = app_state.subscriptions.write().expect("RwLock poisoned");
    let num_token = parameters.subscription_token.parse::<i32>().unwrap();
    // Find the subscription with the matching token
    if let Some(subscription) = subscriptions.iter_mut().find(|s| s.id == num_token) {
        // Update the status to "confirmed"
        subscription.status = "confirmed".to_string();
        HttpResponse::Ok().json(json!({
            "message": "Subscription confirmed successfully"
        }))
    } else {
        HttpResponse::NotFound().json(json!({
            "error": "Subscription not found"
        }))
    }
}