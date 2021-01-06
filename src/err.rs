use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub enum AppError {
    Timeout,
    NoConfigDirectory,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AppError::Timeout => write!(f, "Timeout while waiting user action"),
            AppError::NoConfigDirectory => write!(f, "Unable to locate config directory"),
        }
    }
}

impl Error for AppError {}
