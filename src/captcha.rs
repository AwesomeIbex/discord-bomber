use anyhow::Error;
use reqwest::{Client, StatusCode};
use reqwest::header::{CONNECTION, CONTENT_TYPE, HeaderMap, USER_AGENT as USER_AGENT_PARAM};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::discord::{DISCORD_SITE_KEY, DISCORD_REGISTER_URL};
use crate::email::{MAIL_API_URL, USER_AGENT};
use crate::email::EmailUser;

//TODO read me from env
const TWO_CAPTCHA_API_KEY: &str = "";
const TWO_CAPTCHA_URL: &str = "http://2captcha.com/in.php";

pub async fn solve() -> Result<String, Error> {
    log::info!("> Solving captcha..");
    let client = Client::builder()
        .cookie_store(true)
        .build()?;

    let response = client.post(TWO_CAPTCHA_URL)
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

    log::info!("> Received captcha query with results {:?}", response);

    let captcha_id = response[1].clone();

    log::info!("> Extracted captcha id {}", captcha_id);

    let mut answer = check_answer(&client, &captcha_id).await?;

    log::info!("> Checking initial captcha answer {}", answer);

    let mut counter = 0;
    while answer.contains("CAPCHA_NOT_READY") {
        log::info!("> Checking captcha answer for the {} time {}", counter, answer);

        sleep(Duration::from_secs(5)).await;
        answer = check_answer(&client, &captcha_id).await?;
        counter += 1;
    }

    Ok(answer)
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