use anyhow::Error;

use crate::user::User;
use tokio::fs::File;
use tokio::io::{BufWriter, AsyncWriteExt};
use crate::discord::DISCORD_INVITE_LINK;

mod email;
mod discord;
mod captcha;
mod user;

//TODO make main DTO to carry everything

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let mut user = User::new();

    let mut u = vec![];
    let email = email::create(&user).await?;
    log::info!("Created email user, response: {:?}", email);

    let token = email::token(&user).await?;
    log::info!("Retrieved email token, response: {:?}", token);

    user = user.with_email_token(&token.token);

    let captcha_key = captcha::solve().await?;
    log::info!("Retrieved captcha key: {:?}", captcha_key);

    user = user.with_captcha_key(&captcha_key);

    let discord_token = discord::register(captcha_key, &user).await?;
    log::info!("Retrieved discord auth token: {:?}", discord_token);

    user = user.with_discord_token(&discord_token);


    discord::join_server(&user).await?;
    log::info!("Joined discord server at {}", DISCORD_INVITE_LINK);


    u.push(user);

    let file = File::open("accounts.json").await?;
    let mut writer = BufWriter::new(file);

    let string = serde_json::to_value(&u)?;
    log::info!("Writing to file: {}", string);
    let result = writer.write(string.to_string().as_bytes()).await.unwrap();
    // Join server

    // Destroy


    Ok(())
}