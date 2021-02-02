use anyhow::Error;

use crate::user::User;
use tokio::fs::File;
use tokio::io::{BufWriter, AsyncWriteExt};
use crate::discord::DISCORD_INVITE_LINK;
use crate::cli::get_opts_args;

mod email;
mod discord;
mod captcha;
mod user;
mod cli;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let opts = get_opts_args();

    let mut user = User::new(&opts);

    let mut users = vec![];
    let email = email::create(&user).await?;
    log::info!("Created email user, response: {:?}", email);

    let token = email::token(&user).await?;
    log::info!("Retrieved email token, response: {:?}", token);

    user = user.with_email_token(&token.token);

    //TODO test for rate limiting first

    let captcha_key = captcha::solve().await?;
    log::info!("Retrieved captcha key: {:?}", captcha_key);

    user = user.with_captcha_key(&captcha_key);

    //TODO check rate limit BEFORE getting captcha

    let discord_token = discord::register(captcha_key, &user).await?;
    log::info!("Retrieved discord auth token: {:?}", discord_token);

    users.push(user.clone());

    write_to_file(&mut users).await?;

    // user = user.with_discord_token(&discord_token.token);

    log::info!("User updated");

    // discord::join_server(&user).await?;
    log::info!("Joined discord server at {}", DISCORD_INVITE_LINK);

    // Destroy


    Ok(())
}

async fn write_to_file(u: &mut Vec<User>) -> Result<(), Error> {
    let file = File::open("accounts.json").await?;
    let mut writer = BufWriter::new(file);

    let string = serde_json::to_value(&u)?;
    log::info!("Writing to file: {}", string);
    let result = writer.write(string.to_string().as_bytes()).await.unwrap();
    Ok(())
}