use serde::{Deserialize, Serialize};
use validator::Validate;
use reqwest::{Client, Url};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MailjetRequest<'a> {
    pub from_email: &'a str,
    pub from_name: &'a str,
    pub subject: &'a str,
    pub text_part: &'a str,
    pub html_part: &'a str,
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize)]
struct Recipient {
    email: String,
}

//c8a80214b69ec65426d8603f760c3382 APIKEY
// secret key ac4b89b90dc3f4efc50d502d7e24e298
#[derive(Validate)]
pub struct EmailClient{
    http_client: Client,
    base_url: reqwest::Url,
    #[validate(email)]
    sender: String
}

impl EmailClient {
    pub fn new(base_url: String, sender: String) -> Self {
        let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
        Self {
            http_client: http_client,
            base_url: Url::parse(&base_url).unwrap(),
            sender
        }
    }
    
    pub async fn send_email(
    &self,
    recipient: String,
    subject: &str,
    html_content: &str,
    text_content: &str
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}", self.base_url);
        let request = MailjetRequest {
            from_email: self.sender.as_ref(),
            from_name: "Mailjet Pilot".as_ref(),
            subject: subject.as_ref(),
            text_part: text_content.as_ref(),
            html_part: html_content.as_ref(),
            recipients: vec![Recipient{email: recipient}],
        };
        let response = self.http_client
            .post(&url)
            .basic_auth("c8a80214b69ec65426d8603f760c3382", Some("ac4b89b90dc3f4efc50d502d7e24e298"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .error_for_status();


        Ok(())
    }
    }