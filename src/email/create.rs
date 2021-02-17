use anyhow::Error;
use reqwest::header::{CONTENT_TYPE, HeaderMap, USER_AGENT as USER_AGENT_PARAM};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use crate::email::EmailUser;
use crate::email::{MAIL_API_URL, USER_AGENT};
use crate::user::User;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResponse {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub type_field: String,
    #[serde(rename = "id")]
    pub id2: String,
    pub address: String,
    pub quota: i64,
    pub used: i64,
    #[serde(rename = "is_disabled")]
    pub is_disabled: bool,
    #[serde(rename = "created_at")]
    pub created_at: ::serde_json::Value,
    #[serde(rename = "updated_at")]
    pub updated_at: ::serde_json::Value,
}

pub async fn create_email(user: &User) -> Result<CreateResponse, Error> {
    let client = reqwest::Client::builder();
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT_PARAM, USER_AGENT.parse().unwrap());
    header_map.insert("Origin", "https://mail.tm".parse().unwrap());
    header_map.insert("Referer", "https://mail.tm/en".parse().unwrap());
    header_map.insert("TE", "Trailers".parse().unwrap());
    header_map.insert(CONTENT_TYPE, "application/json;charset=utf-8".parse().unwrap()); //TODO memoize me
    let client = client.default_headers(header_map).build()?;

    let create_as_string = serde_json::json!(EmailUser::new(user));
    let string = create_as_string.to_string();
    let res = client.post(format!("{}/accounts", MAIL_API_URL).as_str())
        .body(string)
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
    async fn test_create_user() {
        assert_eq!(create_email(&User::new()).await.unwrap().address.as_str().is_empty(), false)
    }

    #[tokio::test]
    async fn test_create_user_twenty() {
        let mut emails = vec![];
        for x in 0..20 {
            let x: i32 = x;
            let result = create_email(&User::new()).await;
            let response = result.unwrap();
            let string = response.address;
            emails.push(string)
        }
        println!("{:?}", emails);
        assert_eq!(emails.len(), 20)
    }
}