use std::{error::Error, fmt::Display};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::err::AppError;

const CLIENT_ID: &str = "a14deabe89e4f5d2dfb9";
const SCOPE: &str = "notifications";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

mod fs;

pub async fn init_auth_token(client: &Client) -> Result<String, Box<dyn Error>> {
    let config_file_path = fs::get_config_file()?;
    let token = fs::read_token_from_file(&config_file_path)?;

    if token.is_empty() {
        let device_response = get_device_code(&client).await?;
        println!(
            "Please open URL {} in your browser and enter the following code: {}",
            device_response.verification_uri, device_response.user_code
        );
        let token = poll_access_token(
            &client,
            device_response.interval,
            device_response.expires_in,
            &device_response.device_code,
        )
        .await?;
        println!("Access granted! New token: {}", token);
        fs::write_token_to_file(&token, &config_file_path)?;
        Ok(token)
    } else {
        println!("Previous token was found: {}", token);
        Ok(token)
    }
}

#[derive(Deserialize)]
struct DeviceResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Serialize)]
struct DeviceRequest {
    client_id: String,
    scope: String,
}

async fn get_device_code(client: &Client) -> Result<DeviceResponse, Box<dyn Error>> {
    let request_body = DeviceRequest {
        client_id: CLIENT_ID.to_owned(),
        scope: SCOPE.to_owned(),
    };
    let response = client
        .post("https://github.com/login/device/code")
        .json(&request_body)
        .send()
        .await?
        .json::<DeviceResponse>()
        .await?;
    Ok(response)
}

#[derive(Serialize)]
struct AccessTokenRequest {
    client_id: String,
    device_code: String,
    grant_type: String,
}

#[derive(Deserialize)]
struct AccessTokenSuccessResponse {
    access_token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum AccessTokenError {
    AuthorizationPending,
    SlowDown,
    ExpiredToken,
    UnsupportedGrantType,
    IncorrectClientCredentials,
    IncorrectDeviceCode,
    AccessDenied,
}

#[derive(Deserialize, Debug)]
struct AccessTokenErrorResponse {
    error: AccessTokenError,
    error_description: String,
    error_uri: String,
}

impl Display for AccessTokenErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_description)
    }
}

impl Error for AccessTokenErrorResponse {}

#[derive(Deserialize)]
#[serde(untagged)]
enum AccessTokenResponse {
    SuccessfulResponse(AccessTokenSuccessResponse),
    ErrorResponse(AccessTokenErrorResponse),
}

async fn poll_access_token(
    client: &Client,
    mut interval: u64,
    expires_in: u64,
    device_code: &str,
) -> Result<String, Box<dyn Error>> {
    let request_body = AccessTokenRequest {
        client_id: CLIENT_ID.to_owned(),
        device_code: device_code.to_owned(),
        grant_type: GRANT_TYPE.to_owned(),
    };

    let limit: u64 = expires_in / interval;
    let mut count = 0;

    loop {
        count += 1;
        let response = client
            .post("https://github.com/login/oauth/access_token")
            .json(&request_body)
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        match response {
            AccessTokenResponse::SuccessfulResponse(r) => {
                break Ok(r.access_token);
            }
            AccessTokenResponse::ErrorResponse(e) => match e.error {
                AccessTokenError::AuthorizationPending => {
                    if count >= limit {
                        break Err(Box::new(AppError::Timeout));
                    } else {
                        sleep(Duration::from_secs(interval)).await;
                        continue;
                    }
                }
                AccessTokenError::SlowDown => {
                    if count >= limit {
                        break Err(Box::new(AppError::Timeout));
                    } else {
                        interval += 5;
                        sleep(Duration::from_secs(interval)).await;
                        continue;
                    }
                }
                _ => {
                    break Err(Box::new(e));
                }
            },
        }
    }
}
