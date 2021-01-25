use crate::user::User;
use anyhow::Error;

mod email;
mod discord;
mod captcha;
mod user;

//TODO make main DTO to carry everything

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let mut user = User::new();

    let email = email::create(&user).await?;
    log::info!("> Created email user, response: {:?}", email);

    let token = email::token(&user).await?;
    log::info!("> Retrieved email token, response: {:?}", token);

    user = user.with_email_token(&token.token);

    let captcha_key = captcha::solve().await?;
    log::info!("> Retrieved captcha key: {:?}", captcha_key);

    user = user.with_captcha_key(&captcha_key);

    let discord_token = discord::register(captcha_key, &user).await?;
    log::info!("> Retrieved discord auth token: {:?}", discord_token);

    user = user.with_discord_token(&discord_token);

    // Join server

    // Destroy


    Ok(())

}