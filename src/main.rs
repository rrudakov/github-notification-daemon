use err::AppError;
use reqwest::Client;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

mod auth;
mod err;

const CONFIG_FILE_NAME: &str = ".ghnotifierrc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let config_file_path = get_config_file()?;
    let token = read_token_from_file(&config_file_path)?;

    if token.is_empty() {
        let device_response = auth::get_device_code(&client).await?;
        println!(
            "Please open URL {} in your browser and enter the following code: {}",
            device_response.verification_uri, device_response.user_code
        );
        let token = auth::poll_access_token(
            &client,
            device_response.interval,
            device_response.expires_in,
            &device_response.device_code,
        )
        .await?;
        println!("Access granted! New token: {}", token);
        write_token_to_file(&token, &config_file_path)?;
    } else {
        println!("Prevoius token was found {}", token);
    }

    Ok(())
}

fn read_token_from_file(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut config_file = OpenOptions::new().read(true).open(path)?;
    let mut token = String::new();
    config_file.read_to_string(&mut token)?;
    Ok(token)
}

fn write_token_to_file(token: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut config_file = OpenOptions::new().write(true).create(true).open(path)?;
    config_file.write_all(token.as_bytes())?;
    Ok(())
}

fn get_config_file() -> Result<PathBuf, Box<dyn Error>> {
    match dirs::config_dir() {
        None => Err(Box::new(AppError::NoConfigDirectory)),
        Some(dir) => {
            let config_file_path = dir.join(CONFIG_FILE_NAME);
            if !config_file_path.exists() {
                File::create(&config_file_path)?;
                Ok(config_file_path)
            } else {
                Ok(config_file_path)
            }
        }
    }
}
