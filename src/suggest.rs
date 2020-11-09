use std::cmp::Reverse;
use std::collections::HashMap;

use regex::Regex;
use scraper::Html;

const IGNORED_WORDS: &'static [&'static str] = &["of", "the", "a", "is", "and", "of", "be", "in",
    "to", "for", "on", "can", "this", "or", "you", "if", "will", "your", "use", "with", "we",
    "from", "our", "it", "by", "that", "an", "how", "when", "create", "here", "used", "mode", "red",
    "hat", "s", "are"];

pub fn get_tags_from_text(document: Html) -> Vec<String> {
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
