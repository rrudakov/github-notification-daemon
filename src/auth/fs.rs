use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use crate::err::AppError;

const CONFIG_FILE_NAME: &str = "ghnotifierrc";

pub fn read_token_from_file(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut config_file = OpenOptions::new().read(true).open(path)?;
    let mut token = String::new();
    config_file.read_to_string(&mut token)?;
    Ok(token)
}

pub fn write_token_to_file(token: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut config_file = OpenOptions::new().write(true).create(true).open(path)?;
    config_file.write_all(token.as_bytes())?;
    Ok(())
}

pub fn get_config_file() -> Result<PathBuf, Box<dyn Error>> {
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
