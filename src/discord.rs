use anyhow::Error;
use reqwest::{Client, StatusCode};
use reqwest::header::{CONNECTION, CONTENT_TYPE, HeaderMap, USER_AGENT as USER_AGENT_PARAM, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::email::{USER_AGENT, EmailUser};
use crate::user::User;

pub const DISCORD_SITE_KEY: &str = "6Lef5iQTAAAAAKeIvIY-DeexoO3gj7ryl9rLMEnn";
pub const DISCORD_REGISTER_URL: &str = "https://discordapp.com/api/v6/auth/register";
pub const DISCORD_LIST_GUILDS: &str = "https://discordapp.com/api/v6/users/@me/guilds";
pub const DISCORD_INVITE_LINK: &str = "VGrH2bnw";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    pub fingerprint: Option<String>,
    pub email: String,
    pub username: String,
    pub password: String,
    pub invite: Option<String>,
    pub consent: bool,
    #[serde(rename = "date_of_birth")]
    pub date_of_birth: String,
    #[serde(rename = "gift_code_sku_id")]
    pub gift_code_sku_id: Option<String>,
    #[serde(rename = "captcha_key")]
    pub captcha_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
}

impl Register {
    fn new(captcha_answer: String, user: &User) -> Register {
        Register {
            fingerprint: None,
            email: user.email.clone(),
            username: user.id.clone(),
            password: user.password.clone(),
            invite: None,
            consent: true,
            date_of_birth: "1990-10-17".to_string(), //TODO randomise me
            gift_code_sku_id: None,
            captcha_key: captcha_answer
        }
    }
}


pub async fn register(captcha_answer: String, user: &User) -> Result<Token, Error> {
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert(CONNECTION, "keep-alive".parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap()); //TODO memoize me

    let client = Client::builder()
        .cookie_store(true)
        .default_headers(header_map)
        .build()?;

    let create_as_string = serde_json::json!(Register::new(captcha_answer, user));
    let res = client.post(DISCORD_REGISTER_URL)
        .body(create_as_string.to_string())
        .send()
        .await?;

    let body = res
        .text()
        .await?;

    log::info!("Received response from discord account creation {}", body);
    //TODO add token to user

    Ok(serde_json::from_str(&body)?)
}

pub async fn check_rate_limit(user: &User) -> Result<Token, Error> {
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert(CONNECTION, "keep-alive".parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap()); //TODO memoize me
    header_map.insert(AUTHORIZATION, user.discord_token.parse().unwrap());

    let client = Client::builder()
        .cookie_store(true)
        .default_headers(header_map)
        .build()?;

    let res = client.get(DISCORD_LIST_GUILDS)
        .send()
        .await?;

    log::info!("Response {:?}", res);

    assert_ne!(res.status().as_u16(), 429);

    let body = res
        .text()
        .await?;

    log::info!("Received response from discord account creation {}", body);
    //TODO add token to user

    Ok(serde_json::from_str(&body)?)
}
pub async fn join_server(user: &User) -> Result<String, Error> {
    log::info!("Joining discord with user {:?}", user);

    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap()); //TODO memoize me
    header_map.insert(AUTHORIZATION, user.discord_token.parse().unwrap());

    let client = Client::builder()
        .cookie_store(true)
        .default_headers(header_map)
        .build()?;

    let res = client.post(format!("https://discordapp.com/api/v6/invite/{}", DISCORD_INVITE_LINK).as_str())
        // .query(&[("with_counts", "true")])
        .send()
        .await?;
    log::info!("Received response from discord joining server {:?}", res);

    let body = res
        .text()
        .await?;

    log::info!("Received body from discord joining server {:?}", body);

    Ok(serde_json::from_str(&body)?)
}

