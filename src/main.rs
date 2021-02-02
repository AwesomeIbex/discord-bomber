use std::fs::read_to_string;
use std::path::Path;

use anyhow::{Context, Error};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::cli::get_opts_args;
use crate::discord::DISCORD_INVITE_LINK;
use crate::user::User;

mod email;
mod discord;
mod captcha;
mod user;
mod cli;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let opts = get_opts_args();

    let mut users = read_users().context("Failed to read users")?;

    log::info!("Users found {:?}", users);
    let mut user = User::new(&opts);

    let email = email::create(&user).await?;
    log::info!("Created email user, response: {:?}", email);

    let token = email::token(&user).await?;
    log::info!("Retrieved email token, response: {:?}", token);

    user = user.with_email_token(&token.token);

    //TODO test for rate limiting first

    let captcha_key = captcha::solve().await?;
    log::info!("Retrieved captcha key: {:?}", captcha_key);

    user = user.with_captcha_key(&captcha_key);

    let discord_token = discord::register(captcha_key, &user).await?;
    log::info!("Retrieved discord auth token: {:?}", discord_token);

    users.push(user.clone());

    write_to_file(&mut users).await.unwrap();

    user = user.with_discord_token(&discord_token.token);

    log::info!("User updated");

    // TODO for each user that hasnt been joined, join the link
    discord::join_server(&user).await?;
    log::info!("Joined discord server at {}", DISCORD_INVITE_LINK);

    users = users.iter()
        .map(|u| {
            if u.id == user.id {
                u.clone().set_joined()
            } else {
                u.clone()
            }
        }).collect();

    // Destroy

    Ok(())
}

fn read_users() -> Result<Vec<User>, Error> {
    let json_file_str = read_to_string(Path::new("accounts.json")).context("file not found")?;
    let users: Vec<User> = serde_json::from_str(&json_file_str).context("error while reading json")?;
    Ok(users)
}

async fn write_to_file(u: &mut Vec<User>) -> Result<(), Error> {
    let mut file = tokio::fs::File::create("./accounts.json").await?;
    let string = serde_json::to_string(&u)?;
    log::debug!("Writing to file: {}", string);
    file.write_all(string.as_bytes()).await?;
    file.sync_all().await?;
    Ok(())
}