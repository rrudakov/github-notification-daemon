use std::error::Error;

use chrono::{DateTime, Utc};
use notify_rust::Hint;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
    Client, Response, Url,
};
use serde::Deserialize;
use tokio::time::{sleep, Duration};

use crate::conf;

#[derive(Deserialize)]
struct Subject {
    title: String,
    latest_comment_url: Option<String>,
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

pub async fn get_notification(token: &str) -> Result<(), Box<dyn Error>> {
    let mut poll_interval;
    let mut since_date_time: Option<DateTime<Utc>> = None;

    loop {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static(conf::ACCEPT_VALUE));
        let client = Client::builder()
            .user_agent(conf::USER_AGENT_VALUE)
            .default_headers(headers)
            .build()?;
        let url = Url::parse(conf::GITHUB_API_BASE_URL)?.join("notifications")?;
        let query = if let Some(date_time) = since_date_time {
            vec![("since", date_time.format("%Y-%m-%dT%H:%M:%SZ").to_string())]
        } else {
            vec![]
        };

        since_date_time = Some(Utc::now());

        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("token {}", token))
            .query(&query)
            .send()
            .await?;

        poll_interval = extract_poll_interval(&response).unwrap_or(60);
        let notifications = response.json::<Vec<Notification>>().await?;

        println!("Fetched {} new notifications", notifications.len());

        for notification in notifications {
            tokio::spawn(async move {
                display_notification(&notification).await;
            });
        }

        sleep(Duration::from_secs(poll_interval)).await;
    }
}

fn extract_poll_interval(response: &Response) -> Result<u64, Box<dyn Error>> {
    match response.headers().get("X-Poll-Interval") {
        Some(poll_interval) => {
            let poll_interval = poll_interval.to_str()?.parse()?;
            Ok(poll_interval)
        }
        None => Ok(60),
    }
}

async fn display_notification(notification: &Notification) {
    let body = format!(
        "[{}]\n{}",
        &notification.subject.subject_type, &notification.subject.title
    );

    let html_url: String = match fetch_html_url(&notification).await {
        Ok(Some(html_url)) => html_url,
        _ => "".into(),
    };

    notify_rust::Notification::new()
        .summary(&notification.repository.full_name)
        .action("default", "Open url")
        .body(&body)
        .hint(Hint::Resident(true))
        .timeout(0)
        .icon("file:///usr/share/icons/Papirus/32x32/apps/github.svg") // TODO: do not use hardcoded icon
        .show()
        .map_or((), |n| {
            n.wait_for_action(|action| {
                if let "default" = action {
                    open_browser_for_notification(&html_url)
                }
            })
        })
}

#[derive(Deserialize)]
struct Ticket {
    html_url: String,
}

fn open_browser_for_notification(html_url: &str) {
    if !html_url.is_empty() {
        if webbrowser::open(html_url).is_ok() {
            ()
        }
    }
}

async fn fetch_html_url(notification: &Notification) -> Result<Option<String>, Box<dyn Error>> {
    match &notification.subject.latest_comment_url {
        Some(ticket_url) => {
            let mut headers = HeaderMap::new();
            headers.insert(ACCEPT, HeaderValue::from_static(conf::ACCEPT_VALUE));
            let client = Client::builder()
                .user_agent(conf::USER_AGENT_VALUE)
                .default_headers(headers)
                .build()?;
            let html_url = client
                .get(ticket_url)
                .send()
                .await?
                .json::<Ticket>()
                .await?
                .html_url;
            Ok(Some(html_url))
        }
        None => Ok(None),
    }
}
