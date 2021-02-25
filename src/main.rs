#![feature(impl_trait_in_bindings)]

use std::fs::read_to_string;
use std::path::Path;

use anyhow::{Context, Error};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::cli::get_opts_args;
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
    log::trace!("Users found {:?}", users);

    let mut user = User::new(&opts);
    email::create(&user).await?;

    let token = email::token(&user).await?;
    user = user.with_email_token(&token.token);

    check_tor().await?;

    let captcha_key = captcha::solve().await?;
    user = user.with_captcha_key(&captcha_key);

    let discord_token = discord::register(captcha_key, &user).await?;
    user = user.with_discord_token(&discord_token.token);


    users.push(user.clone());
    write_to_file(&mut users).await.unwrap();

    log::info!("User updated");

    // TODO for each user that hasnt joined, join the link
    discord::join_server(&user).await?;

    // discord::spam_rick_roll(&user).await?;
    discord::dm_everybody(&user).await?;

    users = users.iter()
        .map(|u| {
            if u.id == user.id {
                log::info!("Updating {:?} to joined", u);
                u.clone().set_joined()
            } else {
                u.clone()
            }
        }).collect();

    write_to_file(&mut users).await.unwrap();

    Ok(())
}

async fn check_tor() -> Result<(), Error> {
    let tor_client = discord::get_client(None).unwrap();

    let res = tor_client.get("https://check.torproject.org").send().await?;
    println!("Status: {}", res.status());

    let text = res.text().await?;
    let is_tor = text.contains("Congratulations. This browser is configured to use Tor.");
    if !is_tor {
        return Err(anyhow::anyhow!("Isnt tor connected"))
    }
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