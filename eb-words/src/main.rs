use structopt::StructOpt;
use std::collections::{HashSet};
use serde::Serialize;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "o", long = "obscurity", default_value = "40")]
    max_obscurity: usize,

    center_letter: char,

    base_word: String,

    words: Vec<String>
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


fn main() {
    let opt = Opt::from_args();
    let base_word = opt.base_word.chars().collect::<HashSet<_>>();
    let mut words = vec![];
    for obscurity in (10..=opt.max_obscurity).step_by(5) {
        match std::fs::read_to_string(format!("wordlists/english-words.{}", obscurity)) {
            Ok(file) => {
                eprintln!("Level: {}", obscurity);
                for word in file.lines() {
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
        center: opt.center_letter,
        outer: base_word.into_iter().collect(),
        words
    };
    println!("{}", serde_json::to_string(&output).unwrap());
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
