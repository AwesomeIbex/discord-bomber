use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::distributions::Alphanumeric;

mod create;
mod list;
mod inspect;
mod auth;

const MAIL_API_URL: &str = "https://api.mail.tm";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0";


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub address: String,
    pub password: String,
}

impl User {
    pub fn new() -> User {
        User {
            address: format!("{}@baybabes.com", get_random_job_id()), //TODO changeme
            password: String::from("stfutop"),
        }
    }


}

fn get_random_job_id() -> String {
    let string = rand::thread_rng().sample_iter(&Alphanumeric).take(20).collect::<String>();
    string
}