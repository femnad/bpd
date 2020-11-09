extern crate roxmltree;

use roxmltree::Document;

const POSTS_ENDPOINT: &str = "https://api.pinboard.in/v1/posts";

pub fn get_pinboard_suggestions(url: String, auth_token: String) -> Result<Vec<String>, ()> {
    let suggestions_url = format!("{}/suggest?url={}&auth_token={}", POSTS_ENDPOINT,
                                  url, auth_token);
    let suggestions_response = ureq::get(&suggestions_url)
        .call();

    let suggestions_text = suggestions_response.into_string()
        .expect("get suggestions fail");

    let doc = Document::parse(&suggestions_text).expect("fail parse");
    let suggestions = doc.descendants()
        .filter(|node| {node.tag_name().name().eq("recommended")})
        .map(|node| { node.text().expect("text fail").to_string() })
        .collect::<Vec<String>>();

    return Ok(suggestions);
}

pub fn add_pinboard_bookmark(pinboard_auth_token: String, title: String, url: String, tag_list: Vec<String>) {
    let tag_list = tag_list.join(" ");
    let post_url = format!("{}/add?auth_token={}&description={}&url={}&tags={}",
                           POSTS_ENDPOINT, pinboard_auth_token, title, url, tag_list);
    ureq::post(post_url.as_str()).call();
}
