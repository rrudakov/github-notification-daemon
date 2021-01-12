use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT},
    Client,
};
use std::error::Error;

mod auth;
mod conf;
mod err;
mod notifications;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(conf::ACCEPT_VALUE));
    let client = Client::builder()
        .user_agent(conf::USER_AGENT_VALUE)
        .default_headers(headers)
        .build()?;
    let token = auth::init_auth_token(&client).await?;
    notifications::get_notification(&token).await?;

    Ok(())
}
