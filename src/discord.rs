use anyhow::{Context, Error};
use cached::proc_macro::cached;
use reqwest::{Client, StatusCode};
use reqwest::header::{AUTHORIZATION, CONNECTION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT as USER_AGENT_PARAM};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::email::{EmailUser, USER_AGENT};
use crate::user::User;
use discord::model::Message;

pub const DISCORD_SITE_KEY: &str = "6Lef5iQTAAAAAKeIvIY-DeexoO3gj7ryl9rLMEnn";
pub const DISCORD_REGISTER_URL: &str = "https://discordapp.com/api/v6/auth/register";
pub const DISCORD_LIST_GUILDS: &str = "https://discordapp.com/api/v6/users/@me/guilds";
pub const TOPEST_DISCORD_INVITE_LINK: &str = "47PDSBM2";
pub const HABIBI_DISCORD_INVITE_LINK: &str = "QQBb2JcUdF";
pub const MEMES_DISCORD_INVITE_LINK: &str = "TFAq8FZ";
pub const DISCORD_INVITE_LINK: &str = MEMES_DISCORD_INVITE_LINK;

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

#[cached]
fn base_headers() -> HeaderMap<HeaderValue> {
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert(CONNECTION, "keep-alive".parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    header_map
}

#[cached(size = 5, result = true)]
fn get_client(token: Option<String>) -> Result<Client, Error> {
    let mut header_map = base_headers();

    if token.is_some() {
        header_map.insert(AUTHORIZATION, token.unwrap().parse().unwrap());
    }
    let client = Client::builder()
        .cookie_store(true)
        .default_headers(header_map)
        .build()?;

    Ok(client)
}

impl Register {
    fn new(captcha_answer: String, user: &User) -> Register {
        Register {
            fingerprint: None,
            email: user.email.clone(),
            username: strip_max_length(user.id.clone()),
            password: user.password.clone(),
            invite: None,
            consent: true,
            date_of_birth: "1990-10-17".to_string(), //TODO randomise me
            gift_code_sku_id: None,
            captcha_key: captcha_answer,
        }
    }
}

fn strip_max_length(id: String) -> String {
    let mut id = id;
    if id.len() > 31 {
        id.replace_range(32..id.len(), "");
    }
    id
}


pub async fn register(captcha_answer: String, user: &User) -> Result<Token, Error> {
    let client = get_client(None)?;

    let create_as_string = serde_json::json!(Register::new(captcha_answer, user));
    let res = client.post(DISCORD_REGISTER_URL)
        .body(create_as_string.to_string())
        .send()
        .await?;

    let body = res
        .text()
        .await?;

    log::info!("Received response from discord account creation {}", body);

    let token = serde_json::from_str(&body)?;
    log::info!("Retrieved discord auth token: {:?}", token);
    Ok(token)
}

pub async fn check_rate_limit(user: &User) -> Result<Token, Error> {
    let client = get_client(Some(user.discord_token.to_string()))?;

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

pub async fn spam_rick_roll(user: &User) -> Result<String, Error> {
    log::info!("Instantiating client");
    let client = discord::Discord::from_user_token(&user.discord_token)?;
    log::info!("Getting servers");
    let servers = client.get_servers()?;
    log::info!("Searching for first server");
    let server = servers
        .first()
        .context("Failed to find any working server")?;

    log::info!("Getting channels");
    let channels = client.get_server_channels(server.id)?;
    log::info!("Getting first channel");
    let channel = channels
        .first()
        .context("Failed to find any channels for this server")?;

    log::info!("Sending rick roll");

    let client = get_client(Some(user.discord_token.to_string()))?;
    let create_as_string = r#"{"content":""https://www.youtube.com/watch?v=dQw4w9WgXcQ","nonce":"811750359658659840","tts":false}"#;
    let res = client.post(&format!("https://discord.com/api/v8/channels/{}/messages", channel.id.to_string()))
        .body(create_as_string.to_string())
        .send()
        .await?;

    let body = res
        .text()
        .await?;
    log::info!("{}", body);

    Ok(body)
}

pub async fn join_server(user: &User) -> Result<String, Error> {
    log::info!("Joining discord with user {:?}", user);

    let client = get_client(Some(user.discord_token.to_string()))?;

    let res = client.post(format!("https://discordapp.com/api/v6/invite/{}", DISCORD_INVITE_LINK).as_str())
        .send()
        .await?;
    log::info!("Received response from discord joining server {:?}", res);

    let body = res
        .text()
        .await?;

    log::info!("Received body from discord joining server {:?}", body);

    log::info!("Joined discord server at {}", DISCORD_INVITE_LINK);

    Ok(body)
}

