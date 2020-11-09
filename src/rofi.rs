use std::process::{Command, Stdio};
use std::io::{Write, Read};

pub fn select_tags(url: String, tag_list: Vec<String>) -> Vec<String> {
    let tag_list = tag_list.join("\n");
    let rofi = Command::new("rofi")
        .args(vec!["-dmenu", "-multi-select", "-sep", " ", "-p", "Add Pinboard Bookmark",
                   "-mesg", &url])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn fail");

    rofi.stdin.unwrap().write_all(tag_list.as_bytes()).unwrap();

    let mut tag_list = String::new();
    rofi.stdout.unwrap().read_to_string(&mut tag_list).unwrap();
    let tags: Vec<&str> = tag_list.split('\n').collect();
    return tags.iter().map(|s| s.to_string()).collect();
}
