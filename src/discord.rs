use std::future::Future;

use anyhow::{Context, Error};
use cached::proc_macro::cached;
use discord::GetMessages;
use discord::model::{ChannelId, Message, PrivateChannel, PublicChannel, User as DiscordUser, UserId};
use futures::SinkExt;
use itertools::Itertools;
use reqwest::{Client, Response, StatusCode};
use reqwest::header::{AUTHORIZATION, CONNECTION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT as USER_AGENT_PARAM};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::email::{EmailUser, USER_AGENT};
use crate::user::User;

pub const DISCORD_SITE_KEY: &str = "6Lef5iQTAAAAAKeIvIY-DeexoO3gj7ryl9rLMEnn";
pub const DISCORD_REGISTER_URL: &str = "https://discordapp.com/api/v6/auth/register";
pub const DISCORD_LIST_GUILDS: &str = "https://discordapp.com/api/v6/users/@me/guilds";
pub const TOPEST_DISCORD_INVITE_LINK: &str = "47PDSBM2";
pub const HABIBI_DISCORD_INVITE_LINK: &str = "QQBb2JcUdF";
pub const MEMES_DISCORD_INVITE_LINK: &str = "TFAq8FZ";
pub const DISCORD_INVITE_LINK: &str = HABIBI_DISCORD_INVITE_LINK;

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessage {
    pub id: String,
    pub author: Author,
    pub mentions: Vec<Mention>,
    pub pinned: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    pub id: String,
    pub username: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub discriminator: String,
    #[serde(rename = "public_flags")]
    pub public_flags: i64,
    pub bot: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDm {
    pub recipients: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dm {
    pub content: String,
    pub nonce: String,
    pub tts: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelResponse {
    pub id: String,
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
pub(crate) fn get_client(token: Option<String>) -> Result<Client, Error> {
    let mut header_map = base_headers();

    let proxy = reqwest::Proxy::all("socks5://127.0.0.1:9150").expect("tor proxy should be there");

    if token.is_some() {
        header_map.insert(AUTHORIZATION, token.unwrap().parse().unwrap());
    }
    let client = Client::builder()
        .proxy(proxy)
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

pub async fn dm_everybody(user: &User) -> Result<Vec<Message>, Error> {
    log::info!("Instantiating client");
    let client = discord::Discord::from_user_token(&user.discord_token)?;
    log::info!("Getting servers");
    let servers = client.get_servers()?;
    log::info!("Searching for first server from {:?}", servers);
    let server_id = servers
        .first()
        .context("Failed to find any working server")?
        .id;

    let channel_id = client.get_server_channels(server_id)?
        .iter()
        .filter(|channel| channel.name.eq("general") || channel.name.eq("introductions"))
        .cloned()
        .collect::<Vec<PublicChannel>>()[0].id;

    log::info!("Getting initial client");
    let rest_client = get_client(Some(user.discord_token.to_string()))?;

    let mut messages: Vec<Message> = vec![];
    for i in 0..3 {
        // curl 'https://discord.com/api/v8/channels/793841573187813379/messages?limit=50' -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0' -H 'Accept: */*' -H 'Accept-Language: en-GB' --compressed -H 'Authorization: mfa.fej_DpEjur3nxEalyu1Me_OL2KzQhCRS6zDOezKZAWwuazPvC1edoXCOoLcMcT3HJggEx3nSTyFA1_bwzN_a' -H 'X-Super-Properties: eyJvcyI6IkxpbnV4IiwiYnJvd3NlciI6IkZpcmVmb3giLCJkZXZpY2UiOiIiLCJzeXN0ZW1fbG9jYWxlIjoiZW4tVVMiLCJicm93c2VyX3VzZXJfYWdlbnQiOiJNb3ppbGxhLzUuMCAoWDExOyBMaW51eCB4ODZfNjQ7IHJ2Ojg3LjApIEdlY2tvLzIwMTAwMTAxIEZpcmVmb3gvODcuMCIsImJyb3dzZXJfdmVyc2lvbiI6Ijg3LjAiLCJvc192ZXJzaW9uIjoiIiwicmVmZXJyZXIiOiIiLCJyZWZlcnJpbmdfZG9tYWluIjoiIiwicmVmZXJyZXJfY3VycmVudCI6IiIsInJlZmVycmluZ19kb21haW5fY3VycmVudCI6IiIsInJlbGVhc2VfY2hhbm5lbCI6InN0YWJsZSIsImNsaWVudF9idWlsZF9udW1iZXIiOjc3NjQ1LCJjbGllbnRfZXZlbnRfc291cmNlIjpudWxsfQ==' -H 'X-Fingerprint: 814220013564198944.5VYuUVcapWJ6DY7KfGJfJXoacoA' -H 'Alt-Used: discord.com' -H 'Connection: keep-alive' -H 'Referer: https://discord.com/channels/793832870674169878/804062298653458444' -H 'Cookie: __cfduid=d2d30f09bdbe2b2bb55dde607dbc433501611773648; _ga=GA1.2.1192072284.1611773650; locale=en-GB' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' -H 'TE: Trailers'
        // curl 'https://discord.com/api/v8/channels/793841573187813379/messages?before=814202986329931857&limit=50' -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0' -H 'Accept: */*' -H 'Accept-Language: en-GB' --compressed -H 'Authorization: mfa.fej_DpEjur3nxEalyu1Me_OL2KzQhCRS6zDOezKZAWwuazPvC1edoXCOoLcMcT3HJggEx3nSTyFA1_bwzN_a' -H 'X-Super-Properties: eyJvcyI6IkxpbnV4IiwiYnJvd3NlciI6IkZpcmVmb3giLCJkZXZpY2UiOiIiLCJzeXN0ZW1fbG9jYWxlIjoiZW4tVVMiLCJicm93c2VyX3VzZXJfYWdlbnQiOiJNb3ppbGxhLzUuMCAoWDExOyBMaW51eCB4ODZfNjQ7IHJ2Ojg3LjApIEdlY2tvLzIwMTAwMTAxIEZpcmVmb3gvODcuMCIsImJyb3dzZXJfdmVyc2lvbiI6Ijg3LjAiLCJvc192ZXJzaW9uIjoiIiwicmVmZXJyZXIiOiIiLCJyZWZlcnJpbmdfZG9tYWluIjoiIiwicmVmZXJyZXJfY3VycmVudCI6IiIsInJlZmVycmluZ19kb21haW5fY3VycmVudCI6IiIsInJlbGVhc2VfY2hhbm5lbCI6InN0YWJsZSIsImNsaWVudF9idWlsZF9udW1iZXIiOjc3NjQ1LCJjbGllbnRfZXZlbnRfc291cmNlIjpudWxsfQ==' -H 'X-Fingerprint: 814220013564198944.5VYuUVcapWJ6DY7KfGJfJXoacoA' -H 'Alt-Used: discord.com' -H 'Connection: keep-alive' -H 'Referer: https://discord.com/channels/793832870674169878/793841573187813379' -H 'Cookie: __cfduid=d2d30f09bdbe2b2bb55dde607dbc433501611773648; _ga=GA1.2.1192072284.1611773650; locale=en-GB' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache'
        let mut res = if messages.is_empty() {
            log::info!("Listing first 100 messages in channel {}", channel_id);
            client.get_messages(channel_id, GetMessages::MostRecent, Some(100))?
        } else {
            log::info!("Listing next 100 messages in channel {}", channel_id);
            client.get_messages(channel_id, GetMessages::Before(messages.last().unwrap().clone().id), Some(100))?
        };

        messages.append(&mut res);
    }

    log::info!("Building mention ids");
    let mut mention_ids = messages
        .iter()
        .flat_map(|msg| msg.mentions.iter().map(|mnt| mnt).collect::<Vec<&DiscordUser>>())
        .cloned()
        .collect::<Vec<DiscordUser>>();
    log::info!("Building user ids");
    let mut user_ids = messages
        .iter()
        .map(|msg| msg.author.clone())
        .collect::<Vec<DiscordUser>>();

    log::info!("Concatenating");
    mention_ids.append(&mut user_ids);

    log::info!("Sanitising");
    let ids = mention_ids
        .iter()
        .unique_by(|user| user.id.0)
        .cloned()
        .collect::<Vec<DiscordUser>>();


    log::info!("Opening initial dm channels");
    let mut results = vec![];
    for id in ids {
        log::info!("Opening initial dm channel with {:?}", id);
        if !id.bot {
            let result = client.create_dm(id.id);
            match result {
                Ok(res) => results.push(res),
                Err(err) => log::error!("Wat {}", err)
            }
        }
    }

    log::info!("Deserialising results");
    let mut channels = vec![];
    for x in results {
        channels.push(x.id);
    }

    log::info!("Building requests to dm...");
    let mut results = vec![];
    for channel in channels {
        log::info!("Building dm for channel {}", channel.0);
        results.push(client.send_message(channel_id, "https://www.youtube.com/watch?v=dQw4w9WgXcQ RICK ROLL", "9128217u83", false));
    }

    for x in results {
        log::info!("Result: {:?}", x)
    }
    // JADE: 595354632037859371
    // TOP: 147510061143425024
    // JJ: 346291025381294082

    // curl 'https://discord.com/api/v8/users/@me/channels' -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0' -H 'Accept: */*' -H 'Accept-Language: en-GB' --compressed -H 'Content-Type: application/json' -H 'X-Context-Properties: e30=' -H 'Authorization: mfa.fej_DpEjur3nxEalyu1Me_OL2KzQhCRS6zDOezKZAWwuazPvC1edoXCOoLcMcT3HJggEx3nSTyFA1_bwzN_a' -H 'X-Super-Properties: eyJvcyI6IkxpbnV4IiwiYnJvd3NlciI6IkZpcmVmb3giLCJkZXZpY2UiOiIiLCJzeXN0ZW1fbG9jYWxlIjoiZW4tVVMiLCJicm93c2VyX3VzZXJfYWdlbnQiOiJNb3ppbGxhLzUuMCAoWDExOyBMaW51eCB4ODZfNjQ7IHJ2Ojg3LjApIEdlY2tvLzIwMTAwMTAxIEZpcmVmb3gvODcuMCIsImJyb3dzZXJfdmVyc2lvbiI6Ijg3LjAiLCJvc192ZXJzaW9uIjoiIiwicmVmZXJyZXIiOiIiLCJyZWZlcnJpbmdfZG9tYWluIjoiIiwicmVmZXJyZXJfY3VycmVudCI6IiIsInJlZmVycmluZ19kb21haW5fY3VycmVudCI6IiIsInJlbGVhc2VfY2hhbm5lbCI6InN0YWJsZSIsImNsaWVudF9idWlsZF9udW1iZXIiOjc3NjQ1LCJjbGllbnRfZXZlbnRfc291cmNlIjpudWxsfQ==' -H 'X-Fingerprint: 814220013564198944.5VYuUVcapWJ6DY7KfGJfJXoacoA' -H 'Origin: https://discord.com' -H 'Alt-Used: discord.com' -H 'Connection: keep-alive' -H 'Referer: https://discord.com/channels/793832870674169878/793841573187813379' -H 'Cookie: __cfduid=d2d30f09bdbe2b2bb55dde607dbc433501611773648; _ga=GA1.2.1192072284.1611773650; locale=en-GB' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' -H 'TE: Trailers' --data-raw '{"recipients":["706231967829590157"]}'
    // the above is the handshake and u get a response like this with new channel id
    // {"id": "814236941519683584", "type": 1, "last_message_id": null, "recipients": [{"id": "706231967829590157", "username": "quima", "avatar": "9b2c46d4f9a6e62f444f80ef99e90131", "discriminator": "3290", "public_flags": 0}]}

    // Then you just post to messages like that
    //curl 'https://discord.com/api/v8/channels/780477491365675048/messages' -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0' -H 'Accept: */*' -H 'Accept-Language: en-GB' --compressed -H 'Content-Type: application/json' -H 'Authorization: mfa.fej_DpEjur3nxEalyu1Me_OL2KzQhCRS6zDOezKZAWwuazPvC1edoXCOoLcMcT3HJggEx3nSTyFA1_bwzN_a' -H 'X-Super-Properties: eyJvcyI6IkxpbnV4IiwiYnJvd3NlciI6IkZpcmVmb3giLCJkZXZpY2UiOiIiLCJzeXN0ZW1fbG9jYWxlIjoiZW4tVVMiLCJicm93c2VyX3VzZXJfYWdlbnQiOiJNb3ppbGxhLzUuMCAoWDExOyBMaW51eCB4ODZfNjQ7IHJ2Ojg3LjApIEdlY2tvLzIwMTAwMTAxIEZpcmVmb3gvODcuMCIsImJyb3dzZXJfdmVyc2lvbiI6Ijg3LjAiLCJvc192ZXJzaW9uIjoiIiwicmVmZXJyZXIiOiIiLCJyZWZlcnJpbmdfZG9tYWluIjoiIiwicmVmZXJyZXJfY3VycmVudCI6IiIsInJlZmVycmluZ19kb21haW5fY3VycmVudCI6IiIsInJlbGVhc2VfY2hhbm5lbCI6InN0YWJsZSIsImNsaWVudF9idWlsZF9udW1iZXIiOjc3NjQ1LCJjbGllbnRfZXZlbnRfc291cmNlIjpudWxsfQ==' -H 'X-Fingerprint: 814220013564198944.5VYuUVcapWJ6DY7KfGJfJXoacoA' -H 'Origin: https://discord.com' -H 'Alt-Used: discord.com' -H 'Connection: keep-alive' -H 'Referer: https://discord.com/channels/@me/780477491365675048' -H 'Cookie: __cfduid=d2d30f09bdbe2b2bb55dde607dbc433501611773648; _ga=GA1.2.1192072284.1611773650; locale=en-GB' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' -H 'TE: Trailers' --data-raw '{"content":"test","nonce":"814236163790340096","tts":false}'

    Ok(messages)
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

