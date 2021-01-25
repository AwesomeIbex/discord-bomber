use rand::Rng;
use rand::distributions::Alphanumeric;

pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub auth_token: String,
    pub email_token: String,
    pub discord_token: String,
    pub captcha_key: String,
}

impl User {
    pub fn new() -> User {
        let id = get_random_job_id();
        User {
            id,
            email: format!("{}@baybabes.com", id), //TODO generate domains
            password: String::from("%q+zsQ4-"),
            auth_token: "".to_string(),
            captcha_key: "".to_string(),
            email_token: "".to_string(),
            discord_token: "".to_string()
        }
    }
    pub fn with_auth_token(self, auth_token: &String) -> User {
        User {
            auth_token: auth_token.to_string(),
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

    fn get_random_job_id() -> String {
        let string = rand::thread_rng().sample_iter(&Alphanumeric).take(20).collect::<String>();
        string
    }
}