use crate::user::User;
use anyhow::Error;

mod email;
mod discord;
mod captcha;
mod user;

//TODO make main DTO to carry everything

#[tokio::main]
async fn main() -> Result<(), Error> {
    // First create user
    let mut user = User::new();

    // then email from user
    let email = email::create(&user).await?;

    // Get an email token to check for verification emails
    let token = email::token(&user).await?;

    // Update base user with token
    user = user.with_email_token(&token.token);

    // Get a captcha key
    let captcha_key = captcha::solve().await?;

    user = user.with_captcha_key(&captcha_key);

    // Register with discord
    let discord_token = discord::register(captcha_key, &User).await?;

    user = user.with_discord_token(&discord_token);

    // Join server

    // Destroy


    Ok(())

}