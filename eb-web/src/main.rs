use rand::prelude::SliceRandom;
use std::collections::HashSet;
use yew::prelude::*;
use yew::services::keyboard::KeyListenerHandle;
use yew::services::KeyboardService;
use yew::web_sys;
use yew::web_sys::Storage;
use serde::Deserialize;

enum Msg {
    PushLetter(char),
    Backspace,
    Submit,
    Shuffle,
    OtherKeypress,
}

#[derive(Deserialize)]
struct Wordlist {
    center: char,
    outer: String,
    words: Vec<String>
}

const TODAY: &str = include_str!("../word-lists/today.json");

struct SpellingBee {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    letters: Vec<char>,
    center: char,
    found_words: Vec<String>,
    current_word: String,
    handle: KeyListenerHandle,
    wordlist: HashSet<String>,
    local_storage: Storage,
    message: Option<String>
}

impl SpellingBee {
    fn callback_for<T>(&self, letter: char) -> Callback<T> {
        self.link.callback(move |_| Msg::PushLetter(letter))
    }

}

fn key(c: char, letters: &[char]) -> String {
    let mut s = String::new();
    s.push(c);
    for c in letters {
        s.push(*c);
    }
    s
}

fn wrap(html: Html) -> Html {
    html! {
        <div class="pz-content">
            <div class="pz-section" id="spelling-bee-container">
                <div class="pz-row pz-game-title-bar">
                    <div class="pz-module" id="portal-game-header">
                        <h2><em class="pz-game-title">{ "Not Spelling Bee" }</em><span class="pz-game-date">{ "November 6, 2021" }</span></h2>
                        <div class="pz-byline"><span class="pz-byline__text">{ "Not Edited by Anyone" }</span></div>
                    </div>
                </div>
                <div id="pz-game-root" class="pz-game-field"> { html }</div>
            </div>
        </div>
    }
}


impl Component for SpellingBee {
    type Message = Msg;
    type Properties = ();
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

        let handle = KeyboardService::register_key_down(
            &yew::utils::window(),
            link.callback(|e: KeyboardEvent| match e.key().as_str() {
                " " => Msg::Shuffle,
                ch if ch.len() == 1 => Msg::PushLetter(ch.chars().next().unwrap()),
                "Backspace" => Msg::Backspace,
                "Enter" => Msg::Submit,
                _ => Msg::OtherKeypress,
            }),
        );
        let today: Wordlist = serde_json::from_str(TODAY).unwrap();
        let letters: Vec<char> =  today.outer.chars().collect();
        let words: String = local_storage.get_item(&key(today.center, &letters)).unwrap().unwrap_or_default();
        let found_words = words
            .lines()
            .map(|line| line.to_owned())
            .collect::<Vec<_>>();
        Self {
            link,
            letters,
            center: today.center,
            found_words,
            current_word: String::new(),
            handle,
            wordlist: today.words.into_iter().collect(),
            local_storage,
            message: None
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PushLetter(c) => self.current_word.push(c),
            Msg::Shuffle => self.letters.shuffle(&mut rand::thread_rng()),
            Msg::Backspace => {
                self.current_word.pop();
            }
            Msg::Submit => {
                if !self.wordlist.contains(&self.current_word) {
                    self.message = Some("Not in wordlist".into());
                    self.current_word.clear();
                } else if self.found_words.contains(&self.current_word){
                    self.message = Some("Already found".into());
                    self.current_word.clear();
                } else {
                    self.found_words
                    .push(std::mem::take(&mut self.current_word))
                }
                self.local_storage
                    .set_item(&key(self.center, &self.letters), &self.found_words.join("\n")).unwrap();
            }
            Msg::OtherKeypress => (),
        };
        true
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let letters = self.letters.iter().map(|letter| {
            html! {
                <svg onclick=self.callback_for(*letter) class="hive-cell outer" viewBox="0 0 120 103.92304845413263">
                    <polygon class="cell-fill" points="0,51.96152422706631 30,0 90,0 120,51.96152422706631 90,103.92304845413263 30,103.92304845413263" stroke="white" stroke-width="7.5">
                    </polygon>
                    <text class="cell-letter" x="50%" y="50%" dy="0.35em">{ letter }</text>
                </svg>

            }
        }).collect::<Html>();
        let center = html! {
                <svg onclick=self.callback_for(self.center) class="hive-cell center" viewBox="0 0 120 103.92304845413263">
                    <polygon class="cell-fill" points="0,51.96152422706631 30,0 90,0 120,51.96152422706631 90,103.92304845413263 30,103.92304845413263" stroke="white" stroke-width="7.5">
                    </polygon>
                    <text class="cell-letter" x="50%" y="50%" dy="0.35em">{ self.center }</text>
                </svg>

        };
        let message = match &self.message {
            Some(message) => html! {
                <div class="message-box error-message">{message}</div>
            },
            None => html! { <div class="message-box" /> }
        };
        // let current_word = self
        //     .current_word
        //     .chars()
        //     .map(|ch| {
        //         if ch == self.center {
        //             html! { <span class="sb-input-bright"> { ch } </span> }
        //         } else if  !self.letters.contains(&ch) {
        //                 html! { <span class="sb-input-extra"> { ch } </span> }
        //         } else {
        //             html! { <span> { ch } </span> }
        //         }
        //     })
        //     .collect::<Html>();
        let current_word = self.current_word.clone();
        let words = self
            .found_words
            .iter()
            .map(|word| html! { <li>{word}</li> })
            .collect::<Html>();
        let inner = html! {
            <div class="sb-content-box">
                <div class="sb-status-box">
                    <div class="sb-wordlist-box">
                        <div class="sb-wordlist-heading">
                            <div class="sb-wordlist-summary">{ format!("You have found {} words", self.found_words.len()) }</div>
                        </div>
                        <div class="sb-wordlist-drawer">
                            <div class="sb-wordlist-window">
                                <div class="sb-wordlist-pag">
                                    <div class="sb-wordlist-scroll-anchor" style="left: 0%;"></div>
                                    <ul class="sb-wordlist-items-pag single">
                                        {words}
                                    </ul>
                                </div>
                                <div class="sb-kebob"></div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="sb-controls">
                    {{ message }}
                    <div class="sb-hive-input">
                        <span class="sb-hive-input-content non-empty" style="font-size: 1em;">
                            <input type="text" value={current_word}/>
                        </span>
                    </div>
                    <div class="sb-hive">
                        <div class="hive">
                            {{ center }}
                            {{ letters }}
                        </div>
                    </div>
                    <div class="hive-actions">
                        <div onclick=self.link.callback(|_|Msg::Submit) class="hive-action hive-action__submit sb-touch-button">{ "Enter" }</div>
                        <div onclick=self.link.callback(|_|Msg::Backspace) class="hive-action hive-action__delete sb-touch-button">{"Delete"}</div>
                    </div>
                </div>
            </div>
        };
        wrap(inner)
    }
}

fn main() {
    yew::start_app::<SpellingBee>();
}
