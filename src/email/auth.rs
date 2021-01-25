use crate::email::create::CreateResponse;
use reqwest::header::{CONTENT_TYPE, HeaderMap, USER_AGENT as USER_AGENT_PARAM};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::email::{MAIL_API_URL, USER_AGENT, User};
use anyhow::Error;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    pub id: String,
}

pub async fn get_token(user: User) -> Result<Token, Error> {
    let client = reqwest::Client::builder();
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert("Origin", "https://mail.tm".parse().unwrap());
    header_map.insert("Referer", "https://mail.tm/en".parse().unwrap());
    header_map.insert("TE", "Trailers".parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json;charset=utf-8".parse().unwrap()); //TODO memoize me
    let client = client.default_headers(header_map).build()?;

    let create_as_string = serde_json::json!(user);
    let res = client.post(format!("{}/authentication_token", MAIL_API_URL).as_str())
        .body(create_as_string.to_string())
        .send()
        .await?
        .text()
        .await?;

    Ok(serde_json::from_str(&res)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_token() {
        assert_eq!(get_token(User::new()).await.unwrap().token.is_empty(), false)
    }
}