extern crate scraper;
extern crate structopt;
extern crate ureq;

use std::process::Command;

use scraper::Html;
use structopt::StructOpt;

mod pinboard;
mod pocket;
mod rofi;
mod suggest;


#[derive(Debug, StructOpt)]
#[structopt(name = "bpd: Post Pinboard bookmarks", about = "Post a bookmark to Pinboard with the selected tags.")]
struct Opt {
    #[structopt(short = "u", long = "url")]
    url: String,
    #[structopt(short = "s", long = "secret")]
    pass_secret: String,
}

fn get_auth_token(secret: String) -> String {
    let pass = Command::new("pass").arg(secret).output().unwrap();
    let lines = String::from_utf8(pass.stdout).expect("output fail");
    let v: Vec<&str> = lines.trim().split('\n').collect();
    v[0].to_string()
}

fn get_title_and_document(url: String) -> (String, Html) {
    let html = ureq::get(url.as_str())
        .call()
        .into_string()
        .unwrap();
    let document = scraper::Html::parse_document(html.as_str());
    let selector = scraper::Selector::parse("head title")
        .expect("selector parse fail");
    let title = document.select(&selector).next().unwrap().inner_html();

    return (title, document);
}

fn get_suggestions(url: String, pinboard_auth_token: String, document: Html) -> Vec<String> {
    let pinboard_suggestions = pinboard::get_pinboard_suggestions(url, pinboard_auth_token);
    if pinboard_suggestions.is_ok() {
        return pinboard_suggestions.unwrap();
    }
    return suggest::get_tags_from_text(document);
}

fn main() {

    let opt = Opt::from_args();
    let pinboard_auth = get_auth_token(opt.pass_secret);
    let pocket_auth = pocket::get_pocket_credentials();

    let (title, document) = get_title_and_document(opt.url.clone());
    let tag_list = get_suggestions(opt.url.clone(), pinboard_auth.clone(), document);

    let selected_tags = rofi::select_tags(opt.url.clone(), tag_list);

    pinboard::add_pinboard_bookmark(pinboard_auth, title.clone(), opt.url.clone(), selected_tags.clone());
    pocket::add_article(opt.url, title, selected_tags, pocket_auth);
}
