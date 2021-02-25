use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::distributions::Alphanumeric;
use crate::user::User;
use crate::email::create::CreateResponse;
use anyhow::{Error, Context};
use crate::email::auth::Token;
use tokio::time::Duration;

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
    log::debug!("Retrieved email token, response: {:?}", token);
    Ok(token)
}
pub async fn verify(user: &User) -> Result<bool, Error> {
    log::info!("Verifying user..");
    log::info!("Listing messages..");
    let mut messages = list::list_messages(&user.email_token).await?;
    while messages.hydra_member.len() == 0 {
        log::info!("Listing messages..");
        messages = list::list_messages(&user.email_token).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    log::info!("Getting first email..");
    let option = messages.hydra_member.first();
    let string = option.cloned().context("Failed to get the first email member")?.id;
    log::info!("Inspecting email..");
    let message = inspect::inspect_email(string, &user.email_token).await?;
    log::info!("Extracting link..");
    let link = inspect::extract_link(message)?;
    log::info!("Verifying..");
    let verify = inspect::verify(link).await?;
    Ok(verify)
}