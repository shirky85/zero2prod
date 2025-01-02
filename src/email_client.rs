use serde::Serialize;
use validator::Validate;
use reqwest::{Client, Url};

#[derive(Serialize)]
struct MailjetRequest<'a> {
    #[serde(rename = "FromEmail")]
    pub from_email: &'a str,
    #[serde(rename = "FromName")]
    pub from_name: &'a str,
    #[serde(rename = "Subject")]
    pub subject: &'a str,
    #[serde(rename = "Text-part")]
    pub text_part: &'a str,
    #[serde(rename = "Html-part")]
    pub html_part: &'a str,
    #[serde(rename = "Recipients")]
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize)]
struct Recipient {
    email: String,
}

//c8a80214b69ec65426d8603f760c3382 APIKEY
// secret key ac4b89b90dc3f4efc50d502d7e24e298
#[derive(Validate, Debug)]
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
    
    #[tracing::instrument(
        name = "Sending an email",
        skip(self, html_content, text_content),
        fields(
        %html_content,
        %recipient,
        %subject
        )
    )]
    pub async fn send_email(
        &self,
        recipient: String,
        subject: &str,
        html_content: &str,
        text_content: &str
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}v3/send", self.base_url);
        let request = MailjetRequest {
            from_email: self.sender.as_ref(),
            from_name: "Newsletter Admin".as_ref(),
            subject: subject.as_ref(),
            text_part: text_content.as_ref(),
            html_part: html_content.as_ref(),
            recipients: vec![Recipient{email: recipient}],
        };
        self.http_client
            .post(&url)
            .basic_auth("c8a80214b69ec65426d8603f760c3382", Some("ac4b89b90dc3f4efc50d502d7e24e298"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .error_for_status()?;


        Ok(())
    }
    }

#[cfg(test)]
mod tests {
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use wiremock::matchers::{any, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use claims::assert_err;

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> String {
        SafeEmail().fake()
    }
    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email() )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/send"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let _ = email_client(mock_server.uri())
        .send_email(email(), &subject(), &content(), &content())
        .await;

        //Assert
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        
        Mock::given(any())
            // Not a 200 anymore!
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client(mock_server.uri())
            .send_email(email(), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        
        let response = ResponseTemplate::new(200)
            // 3 minutes!
            .set_delay(std::time::Duration::from_secs(10));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client(mock_server.uri())
            .send_email(email(), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }
}