use std::error::Error;

use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client, Url,
};
use serde::Deserialize;
use tokio::time::{sleep, Duration};

use crate::conf;

#[derive(Deserialize)]
struct Subject {
    title: String,
    #[serde(alias = "type")]
    subject_type: String,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct Notification {
    subject: Subject,
    repository: Repository,
}

pub async fn get_notification(client: &Client, token: &str) -> Result<(), Box<dyn Error>> {
    let mut poll_interval;

    loop {
        let url = Url::parse(conf::GITHUB_API_BASE_URL)?.join("notifications")?;
        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("token {}", token))
            .send()
            .await?;

        poll_interval = response
            .headers()
            .get("X-Poll-Interval")
            .unwrap_or(&HeaderValue::from_static("60"))
            .to_str()
            .unwrap_or("60")
            .parse::<u64>()
            .unwrap_or(60);
        let notifications = response.json::<Vec<Notification>>().await?;

        for notification in notifications {
            println!(
                "{}\n{}\nType: {}\n",
                notification.repository.full_name,
                notification.subject.title,
                notification.subject.subject_type
            );
        }

        sleep(Duration::from_secs(poll_interval)).await;
    }
}
