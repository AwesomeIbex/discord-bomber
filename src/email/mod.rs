use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::distributions::Alphanumeric;
use crate::user::User;
use crate::email::create::CreateResponse;
use anyhow::Error;
use crate::email::auth::Token;

mod create;
mod list;
mod inspect;
mod auth;

pub(crate) const MAIL_API_URL: &str = "https://api.mail.tm";
pub(crate) const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0";


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailUser {
    pub address: String,
    pub password: String,
}

impl EmailUser {
    pub fn new(user: &User) -> EmailUser {
        EmailUser {
            address: user.email.to_string(),
            password: user.password.to_string()
        }
    }
}

pub async fn create(user: &User) -> Result<CreateResponse, Error> {
    log::info!("Creating email user with id: {} and password {}..", user.id, user.password);
    let response = create::create_email(user).await?;
    log::debug!("Created email user, response: {:?}", response);
    Ok(response)
}
pub async fn token(user: &User) -> Result<Token, Error> {
    log::info!("Retrieving user token..");
    let token = auth::get_token(user).await?;
    log::info!("Retrieved email token, response: {:?}", token);
    Ok(token)
}