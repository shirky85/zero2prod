use actix_web::{http::{self, header::{self, HeaderMap, HeaderValue}, StatusCode}, web, HttpRequest, HttpResponse, ResponseError};
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use validator::Validate;
use secrecy::{ExposeSecret, SecretString};
use sha3::Digest;

use crate::{email_client::EmailClient, in_memory::AppState};

use super::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Validation error(s): {0}")]
    ValidationError(String),
    #[error("Failed to send newsletter email")]
    SendEmailError(#[from]reqwest::Error),
    #[error("Missing Authorization header")]
    MissingAuthorizationHeader,
    #[error("Failed to decode Authorization header")]
    DecodeAuthorizationHeaderError(#[from]base64::DecodeError),
    #[error("Failed to parse Authorization header")]
    ParseAuthorizationHeaderError(#[from]std::string::FromUtf8Error),
    #[error("The 'Authorization' header was not a valid UTF8 string.")]
    InvalidAuthorizationHeaderUTFString(#[from]http::header::ToStrError),
    #[error("Unauthorized sender username")]
    UnauthorizedSenderUsernameError,
    #[error("Wrong sender password")]
    WrongSenderPasswordError,
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn create_response_for_auth_error(status: u16) -> HttpResponse {
    let mut response = HttpResponse::new(StatusCode::from_u16(status).unwrap());
    let header_value = HeaderValue::from_str(r#"Basic realm="publish""#)
    .unwrap();
    response
    .headers_mut()
    // actix_web::http::header provides a collection of constants
    // for the names of several well-known/standard HTTP headers
    .insert(header::WWW_AUTHENTICATE, header_value);
    response
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(json!({ "message": errors }))
            }
            PublishError::SendEmailError(_err) => {
                HttpResponse::BadGateway().finish()
            }
            PublishError::MissingAuthorizationHeader => {
                create_response_for_auth_error(401)
            }
            PublishError::DecodeAuthorizationHeaderError(_err) => {
                create_response_for_auth_error(400)
            }
            PublishError::ParseAuthorizationHeaderError(_err) => {
                create_response_for_auth_error(400)
            }
            PublishError::InvalidAuthorizationHeaderUTFString(_err) => {
                create_response_for_auth_error(400)
            }
            PublishError::UnauthorizedSenderUsernameError => {
                create_response_for_auth_error(401)
            }
            PublishError::WrongSenderPasswordError => {
                create_response_for_auth_error(401)
            }
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct NewsletterRequest {
    #[validate(length(min = 5, max = 50))]
    title: String,
    #[validate()]
    content: Content,
}

#[derive(Deserialize, Validate)]
struct Content {
    #[validate(length(min = 10, max = 500))]
    text: String,
    #[validate(length(min = 10, max = 500))]
    html: String,
}

struct Credentials {
    username: String,
    password: SecretString,
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, PublishError> {
    let header = headers
        .get("Authorization")
        .ok_or(PublishError::MissingAuthorizationHeader)?
        .to_str()
        .map_err(|err| PublishError::InvalidAuthorizationHeaderUTFString(err))?;
    // lose the Basic keyword and decode the base64 encoded string
    let parts: Vec<&str> = header.split_whitespace().collect();
    let decoded = base64::engine::general_purpose::STANDARD.decode(parts[1])
        .map_err(PublishError::DecodeAuthorizationHeaderError)?;

    let decoded_str = String::from_utf8(decoded)
        .map_err(PublishError::ParseAuthorizationHeaderError)?;

    let mut creds= decoded_str.splitn(2 ,":");
    Ok(Credentials {
        username: creds.next().unwrap().to_string(),
        password: SecretString::new(Box::from(creds.next().unwrap().to_string()))
    })
}

// #[tracing::instrument(
//     name = "Validating sender credentials",
//     skip(cred, app_state),
// )]
// async fn validate_sender_credentials(username: &str, password: &SecretString, app_state: &AppState) -> Result<(), PublishError> {
//     let senders = app_state.senders.read().expect("RwLock poisoned");
//     let sender = senders
//         .iter().find(|s| s.username == *username);
//     let hashed_value_of_password = format!("{:x}", sha3::Sha3_256::digest(password.expose_secret().as_bytes()));
//     if sender.unwrap().pwd != hashed_value_of_password {
//         return Err(PublishError::WrongSenderPasswordError);
//     }
//     Ok(())
// }
#[tracing::instrument(
    name = "Validating sender credentials",
    skip(username, password, app_state),
)]
async fn validate_sender_credentials(username: &str, password: &SecretString, app_state: &AppState) -> Result<(), PublishError> {
    let senders = app_state.senders.read().expect("RwLock poisoned");
    let sender = senders
        .iter().find(|s| s.username == *username);
    let hashed_value_of_password = format!("{:x}", sha3::Sha3_256::digest(password.expose_secret().as_bytes()));
    if sender.unwrap().pwd != hashed_value_of_password {
        return Err(PublishError::WrongSenderPasswordError);
    }
    Ok(())
}

#[tracing::instrument(
    name = "Publishing a newsletter",
    skip(req, email_client, app_state),
    fields(
        %req.title,
    )
)]
pub async fn publish_newsletter(
    req: web::Json<NewsletterRequest>,
    email_client: web::Data<EmailClient>,
    app_state: web::Data<AppState>,
    request: HttpRequest) -> Result<HttpResponse, PublishError> {
    let _credentials = basic_authentication(request.headers())
        .map_err(|err| {
            tracing::warn!("Failed to authenticate request: {:?}", err);
            err
        })?;
    let app_state_clone = app_state.clone();
    let validation_handle = tokio::spawn(async move {
        validate_sender_credentials(&_credentials.username, &_credentials.password, &app_state_clone).await
    });

    
    match req.validate() {
        Ok(_) => match req.content.validate(){
            Ok(_) => println!("Request for publish passed validation"),
            Err(errors) => {return Err(PublishError::ValidationError(errors.to_string()));}
        },
        Err(errors) => {return Err(PublishError::ValidationError(errors.to_string()));}
    }

    match validation_handle.await {
        Ok(result) => result?,
        Err(_) => return Err(PublishError::UnauthorizedSenderUsernameError),
    }

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
        ).await?;
    }
    

    Ok(HttpResponse::Ok().finish())
}