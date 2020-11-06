extern crate notify_rust;
extern crate regex;
extern crate roxmltree;
extern crate scraper;
extern crate structopt;
extern crate ureq;

use std::cmp::Reverse;
use std::process::{Command, Stdio, exit};
use std::io::{Write, Read};

use regex::Regex;
use notify_rust::Notification;
use roxmltree::Document;
use structopt::StructOpt;
use std::collections::HashMap;
use scraper::Html;

const POSTS_ENDPOINT: &str = "https://api.pinboard.in/v1/posts";
const IGNORED_WORDS: &'static [&'static str] = &["of", "the", "a", "is", "and", "of", "be", "in",
    "to", "for", "on", "can", "this", "or", "you", "if", "will", "your", "use", "with", "we",
    "from", "our", "it", "by", "that", "an", "how", "when", "create", "here", "used", "mode", "red",
    "hat", "s", "are"];

#[derive(Debug, StructOpt)]
#[structopt(name = "bpd: Post Pinboard bookmarks", about = "Post a bookmark to Pinboard with the selected tags.")]
struct Opt {
    #[structopt(short = "u", long = "url")]
    url: String,
    #[structopt(short = "s", long = "secret")]
    pass_secret: String,
}

struct Bookmark {
    title: String,
    tag_list: String,
}

fn get_auth_token(secret: String) -> String {
    let pass = Command::new("pass").arg(secret).output().unwrap();
    let lines = String::from_utf8(pass.stdout).expect("output fail");
    let v: Vec<&str> = lines.trim().split('\n').collect();
    v[0].to_string()
}

fn get_bookmark(url: String, auth_token: String) -> Bookmark {
    let suggestions_url = format!("{}/suggest?url={}&auth_token={}", POSTS_ENDPOINT,
                                  url, auth_token);
    let suggestions_response = ureq::get(&suggestions_url)
        .call();

    let suggestions_status = suggestions_response.status();
    let suggestions_text = suggestions_response.into_string()
        .expect("get suggestions fail");

    let html = ureq::get(url.as_str())
        .call()
        .into_string()
        .unwrap();
    let document = scraper::Html::parse_document(html.as_str());
    let selector = scraper::Selector::parse("head title")
        .expect("selector parse fail");
    let title = document.select(&selector).next().unwrap().inner_html();

    if suggestions_status >= 400 {
        return Bookmark{title, tag_list: get_tags_from_text(document).join(" ")};
    }

    let doc = Document::parse(&suggestions_text).expect("fail parse");
    let suggestions = doc.descendants()
        .filter(|node| {node.tag_name().name().eq("recommended")})
        .map(|node| { node.text().expect("text fail").to_string() })
        .collect::<Vec<String>>();

    return Bookmark{title, tag_list: suggestions.join(" ")};
}

fn get_tags_from_text(document: Html) -> Vec<String> {
    let mut counts = HashMap::new();
    let word_regex = Regex::new("[a-zA-Z'-]+").unwrap();
    for elem in &["p", "h1", "a"] {
        let selector = scraper::Selector::parse(elem).expect("selector parse fail");
        document.select(&selector).into_iter()
            .for_each(|paragraph| {
                paragraph.text().for_each(|paragraph_text| {
                    word_regex.find_iter(paragraph_text)
                        .map(|matched| matched.as_str().to_lowercase())
                        .filter(|word| !IGNORED_WORDS.contains(&word.as_str()))
                        .for_each(|word| {
                            *counts.entry(word.to_string()).or_insert(0) += 1;
                        })
                });
            });
    }
    let mut words_by_frequency : Vec<_> = counts.iter().collect();
    words_by_frequency.sort_by_key(|&(word, count)| (Reverse(count), word));
    return words_by_frequency.iter()
        .map(|&(word, _count)| word.clone())
        .take(10)
        .collect::<Vec<String>>();
}

fn main() {
    let opt = Opt::from_args();
    let auth_token = get_auth_token(opt.pass_secret);
    let suggestions_url = format!("{}/suggest?url={}&auth_token={}", POSTS_ENDPOINT,
                                  opt.url, auth_token);
    let suggestions_response = ureq::get(&suggestions_url)
        .call();

    let suggestions_status = suggestions_response.status();
    let suggestions_text = suggestions_response.into_string()
        .expect("get suggestions fail");

    let bookmark = get_bookmark(opt.url.clone(), auth_token.clone());

    let rofi = Command::new("rofi")
        .args(vec!["-dmenu", "-multi-select", "-sep", " ", "-p", "Add Pinboard Bookmark",
                   "-mesg", &opt.url])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn fail");

    rofi.stdin.unwrap().write_all(bookmark.tag_list.as_bytes()).unwrap();

    let mut tag_list = String::new();
    rofi.stdout.unwrap().read_to_string(&mut tag_list).unwrap();

    let post_url = format!("{}/add?auth_token={}&description={}&url={}&tags={}",
                           POSTS_ENDPOINT, auth_token, bookmark.title, opt.url, bookmark.tag_list);
    let response = ureq::post(post_url.as_str()).call();
    if response.status() >= 400 {
        Notification::new()
            .summary("Error adding bookmark")
            .body(format!("status: {}, body: {}", suggestions_status, suggestions_text).as_str())
            .show()
            .expect("error showing notification");
        exit(1);
    }

    Notification::new()
        .summary("Added new bookmark")
        .body(format!("For url {}", opt.url).as_str())
        .show()
        .expect("error showing notification");
}
