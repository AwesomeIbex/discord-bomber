use anyhow::Error;
use reqwest::{Client, StatusCode};
use reqwest::header::{CONNECTION, CONTENT_TYPE, HeaderMap, USER_AGENT as USER_AGENT_PARAM};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::discord::{DISCORD_SITE_KEY, DISCORD_REGISTER_URL};
use crate::email::{MAIL_API_URL, USER_AGENT};
use crate::email::User;

//TODO read me from env
const TWO_CAPTCHA_API_KEY: &str = "";

async fn solve() -> Result<String, Error> {
    let client = Client::builder()
        .cookie_store(true)
        .build()?;

    let captcha_id = client.post("http://2captcha.com/in.php")
        .query(&[
            ("key", TWO_CAPTCHA_API_KEY),
            ("method", "userrecaptcha"),
            ("googlekey", DISCORD_SITE_KEY),
            ("pageurl", DISCORD_REGISTER_URL),
        ])
        .send()
        .await?
        .text()
        .await?
        .split("|")
        .map(|item| item.to_string())
        .collect::<Vec<String>>();
    let captcha_id = captcha_id[1].clone();

    let mut recaptcha_answer = check_answer(&client, &captcha_id).await?;

    while recaptcha_answer.contains("CAPCHA_NOT_READY") {
        sleep(Duration::from_secs(5)).await;
        recaptcha_answer = check_answer(&client, &captcha_id).await?;
    }

    Ok(recaptcha_answer)
}

async fn check_answer(client: &Client, captcha_id: &str) -> Result<String, Error> {
    Ok(client.post("http://2captcha.com/res.php")
        .query(&[
            ("key", TWO_CAPTCHA_API_KEY),
            ("action", "get"),
            ("id", captcha_id),
        ])
        .send()
        .await?
        .text()
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user() {
    }

}