use rand::{Rng, RngCore};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use crate::cli::Opts;
use std::fs::File;
use csv::Reader;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub email_token: String,
    pub discord_token: String,
    pub captcha_key: String,
    pub joined_server: bool,
}

#[derive(Debug, Deserialize)]
struct WordRecord {
    word: String,
}

impl User {
    pub fn new(opts: &Opts) -> User {
        let id = User::build_id(User::build_word_list());

        User {
            id: id.clone(),
            email: format!("{}@maxresistance.com", id.to_lowercase()), //TODO generate domains
            password: String::from("%q+zsQ4-"),
            captcha_key: "".to_string(),
            email_token: "".to_string(),
            discord_token: "".to_string(),
            joined_server: false
        }
    }

    fn build_word_list() -> Vec<WordRecord> {
        let mut file = File::open("words.csv").expect("Failed to open words"); //TODO fix me
        let mut rdr = csv::Reader::from_reader(file);

        let mut words = vec![];
        for result in rdr.deserialize() {
            let record: WordRecord = result.expect("Failed to read a word record");
            words.push(record);
        }
        words
    }
    fn build_id(words: Vec<WordRecord>) -> String {
        let mut rnd = rand::thread_rng();

        let mut words_collected: Vec<String> = vec![];

        let len = words.len();
        words_collected.push(words.get(rnd.gen_range(0, len)).unwrap().word.clone());
        words_collected.push(words.get(rnd.gen_range(0, len)).unwrap().word.clone());
        words_collected.push(words.get(rnd.gen_range(0, len)).unwrap().word.clone());

        words_collected.join("")
    }
    pub fn set_joined(self) -> User {
        User {
            joined_server: true,
            ..self
        }
    }
    pub fn with_captcha_key(self, captcha_key: &String) -> User {
        User {
            captcha_key: captcha_key.to_string(),
            ..self
        }
    }
    pub fn with_email_token(self, email_token: &String) -> User {
        User {
            email_token: email_token.to_string(),
            ..self
        }
    }
    pub fn with_discord_token(self, discord_token: &String) -> User {
        User {
            discord_token: discord_token.to_string(),
            ..self
        }
    }
}

fn get_random_job_id() -> String {
    let string = rand::thread_rng().sample_iter(&Alphanumeric).take(20).collect::<String>();
    string
}