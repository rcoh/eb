use structopt::StructOpt;
use std::collections::{HashSet};
use std::error::Error;
use serde::Serialize;
use reqwest::Url;
use scraper::{Html, Selector};
use time::macros::offset;
use regex::Regex;
use time::OffsetDateTime;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "o", long = "obscurity", default_value = "50")]
    max_obscurity: usize,

    #[structopt(short = "c", long = "center")]
    center_letter: Option<char>,

    #[structopt(short = "w", long = "word")]
    base_word: Option<String>,

}

fn is_emily_word_for(sb_word: &HashSet<char>, possible: impl Iterator<Item=char>) -> bool {
    let possible = possible.collect::<HashSet::<_>>();
    sb_word.difference(&possible).count() == 1 && possible.difference(&sb_word).count() == 1
}

#[derive(Serialize)]
struct Output {
    center: char,
    outer: String,
    words: Vec<String>
}

async fn scrape(date: &str) -> (char, String) {
    eprintln!("loading page for {}", date);
    let uri: Url = format!("https://www.nytimes.com/{date}/crosswords/spelling-bee-forum.html", date = &date).parse().expect("valid uri");
    let page = reqwest::Client::new().get(uri).send().await.expect("failed to load URI");
    assert!(page.status().is_success());
    let html_str = page.bytes().await.expect("failed to load data");
    let html = Html::parse_document(std::str::from_utf8(html_str.as_ref()).unwrap());
    let selector = Selector::parse("p.css-axufdj").expect("valid selector");
    let re = Regex::new("<p .*?<strong.*?>([A-Z])[ ]*</strong>([A-Z ]+)</p>").unwrap();
    for element in html.select(&selector) {
        let html = element.html();
        if let Some(captures) = re.captures(&html) {
            return (captures.get(1).unwrap().as_str().chars().next().unwrap().to_ascii_lowercase(), captures.get(2).unwrap().as_str().replace(' ', "").to_ascii_lowercase())
        }
    }
    panic!("could not find match")
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let opt = Opt::from_args();
    let today = OffsetDateTime::now_utc().to_offset(offset!(-5));
    let today = format!("{}/{:02}/{:02}", today.year(), u8::from(today.month()), today.day());
    let (center, letters) = scrape(&today).await;

    let mut base_word = letters.chars().collect::<HashSet<_>>();
    base_word.insert(center);
    let mut words = vec![];
    for obscurity in (10..=opt.max_obscurity).step_by(5) {
        match std::fs::read_to_string(format!("wordlists/english-words.{}", obscurity)) {
            Ok(file) => {
                eprintln!("Level: {}", obscurity);
                for word in file.lines() {
                    if !word.chars().all(|c|c.is_alphabetic()) {
                        continue;
                    }
                    let chars = word.trim().chars().filter(|c|c.is_alphabetic());
                    if is_emily_word_for(&base_word, chars) {
                        eprintln!("{}", word);
                        words.push(word.to_string());
                    }
                }
            },
            Err(_no_file) => {
            }
        }
    }
    let output = Output {
        center,
        outer: letters,
        words
    };
    println!("{:?}", std::env::current_dir());
    let today = today.replace("/", "-");
    std::fs::write(format!("../eb-web/word-lists/{}.json", today), serde_json::to_string(&output)?)?;
    std::fs::remove_file("../eb-web/word-lists/today.json")?;
    std::fs::write("../eb-web/word-lists/today.json", serde_json::to_string(&output)?)?;
    println!("done!");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::is_emily_word_for;

    #[test]
    fn emily_words() {
        let base = "gamecock".chars().collect();
        assert!(is_emily_word_for(&base, "lockage"));
        assert!(!is_emily_word_for(&base, "bloop"))
    }

}
