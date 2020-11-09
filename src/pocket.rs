extern crate ureq;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

const AUTH_FILE: &str = ".config/bpd/bpd.json";
const CONSUMER_KEY: &str = "94154-6565f834dc4ba147df581009";
const REDIRECT_URI: &str = "https://getpocket.com/connected_applications";

#[derive(Deserialize)]
struct OauthCode {
    code: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserAuth {
    access_token: String,
    username: String,
}

fn get_auth_file() -> String {
    let home = env::var("HOME").expect("error getting home");
    return format!("{}/{}", home, AUTH_FILE);
}

pub fn get_user_auth() -> Result<UserAuth, ()> {
    let oauth_request_url = format!("https://getpocket.com/v3/oauth/request?consumer_key={}&redirect_uri={}", CONSUMER_KEY, REDIRECT_URI);
    let oauth_response = ureq::post(&oauth_request_url)
        .set("Content-Type", "application/json; charset=UTF-8")
        .set("X-Accept", "application/json")
        .call();

    if oauth_response.status() != 200 {
        println!("Unexpected response in Oauth request: {}", oauth_response.status());
        return Err(());
    }

    let code = oauth_response.into_json_deserialize::<OauthCode>().unwrap();

    println!("Manual redirect to: https://getpocket.com/auth/authorize?request_token={}&redirect_uri={}", code.code, REDIRECT_URI);

    let mut user_input = String::new();
    print!("Continue?");
    stdout().flush().unwrap();
    stdin().read_line(&mut user_input).unwrap();

    let authorize_response = ureq::post("https://getpocket.com/v3/oauth/authorize")
        .set("Content-Type", "application/json; charset=UTF-8")
        .set("X-Accept", "application/json")
        .send_json(json!({
            "consumer_key": CONSUMER_KEY,
            "code": code.code,
    }));

    if authorize_response.status() != 200 {
        println!("Unexpected response in authorize request: {}", authorize_response.status());
        return Err(());
    }

    let user_auth = authorize_response.into_json_deserialize::<UserAuth>().unwrap();
    return Ok(user_auth);

}

pub fn store_pocket_credentials(user_auth: UserAuth) {
    let auth_file_path = get_auth_file();
    let auth_file = Path::new(&auth_file_path);
    std::fs::create_dir_all(auth_file.parent().unwrap()).unwrap();
    let serialized = serde_json::to_string(&user_auth).unwrap();
    let mut file = File::create(auth_file).unwrap();
    write!(file, "{}", serialized).unwrap();
}

pub fn get_pocket_credentials() -> UserAuth {
    let file = File::open(get_auth_file());
    if file.is_err() {
        let user_auth = get_user_auth().expect("error authenticating with Pocket");
        store_pocket_credentials(user_auth.clone());
        return user_auth;
    }
    let user_auth: UserAuth = serde_json::from_reader(file.unwrap()).unwrap();
    return user_auth;
}

pub fn add_article(url: String, title: String, tag_list: Vec<String>, user_auth: UserAuth) {
    let tag_list = tag_list.join(",");
    ureq::post("https://getpocket.com/v3/add")
        .set("Content-Type", "application/json; charset=UTF-8")
        .set("X-Accept", "application/json")
        .send_json(json!({
            "url": url,
            "title": title,
            "tag_list": tag_list,
            "consumer_key": CONSUMER_KEY,
            "access_token": user_auth.access_token,
        }));
}
