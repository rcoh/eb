mod keyboard;

use keyboard::Keyboard;
use gloo_timers::callback::Timeout;
use rand::prelude::SliceRandom;
use serde::Deserialize;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::services::keyboard::KeyListenerHandle;
use yew::services::{ConsoleService, KeyboardService};
use yew::web_sys;
use yew::web_sys::Storage;
use crate::web_sys::console::log;

enum Msg {
    PushLetter(char),
    ToggleWords,
    Backspace,
    Keyboard,
    Submit,
    ClearMessage,
    Shuffle,
    OtherKeypress,
}

#[derive(Deserialize)]
struct Wordlist {
    center: char,
    outer: String,
    words: Vec<String>,
}

impl Wordlist {
    fn to_set(&self) -> HashSet<char> {
        self.outer.chars().chain(Some(self.center)).collect()
    }
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
    wordlist: Wordlist,
    local_storage: Storage,
    message: Option<String>,
    wordlist_visible: bool,
}

impl SpellingBee {
    fn callback_for<T>(&self, letter: char) -> Callback<T> {
        self.link.callback(move |_| Msg::PushLetter(letter))
    }

    fn grid(&self) -> HashSet<char> {
        let mut letters: HashSet<char> = self.letters.iter().cloned().collect();
        letters.insert(self.center);
        letters
    }

    fn purple(&self) -> Option<char> {
        let grid = self.grid();

        let purple = self.current_word.chars().find(|letter|!grid.contains(letter));
        ConsoleService::info(&format!("grid: {:?}, word: {}, pruple: {:?}", &grid, &self.current_word, purple));
        purple
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

/*
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
                <div class="pz-row"><div class="pz-module pz-flex-row pz-game-toolbar-content" id="portal-game-toolbar"><div class="pz-toolbar-left"></div><div class="pz-toolbar-right"><span role="presentation" class="pz-toolbar-button pz-toolbar-button__yesterday">{ "Yesterday’s Answers" }</span><a class="pz-toolbar-button pz-toolbar-button__hints" href="https://www.nytimes.com/2021/11/08/crosswords/spelling-bee-forum.html" target="_blank" rel="noreferrer"><i class="pz-toolbar-icon external"></i></a><div class="pz-dropdown"><button type="button" class="pz-toolbar-button pz-dropdown__button"><span class="pz-dropdown__label"></span><span class="pz-dropdown__arrow"></span></button></div></div></div></div>
                <div id="pz-game-root" class="pz-game-field"> { html }</div>
            </div>
        </div>
    }
}*/

fn error(word_list: &Wordlist, guess: &str) -> String {
    let guess = guess.chars().collect::<HashSet<_>>();
    let rules = word_list.to_set();
    if guess.difference(&rules).count() > 1 {
        format!("Too many new letters")
    } else if rules.difference(&guess).count() > 1 {
        format!(
            "All letters except one must be included. Missing: {:?}",
            rules.difference(&guess).collect::<Vec<_>>()
        )
    } else {
        format!("Not in wordlist")
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
        let letters: Vec<char> = today.outer.chars().collect();
        let words: String = local_storage
            .get_item(&key(today.center, &letters))
            .unwrap()
            .unwrap_or_default();
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
            wordlist: today,
            local_storage,
            message: None,
            wordlist_visible: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleWords => { self.wordlist_visible = !self.wordlist_visible },
            Msg::ClearMessage => self.message = None,
            Msg::PushLetter(c) => {
                self.current_word.push(c.to_ascii_lowercase());
            }
            Msg::Shuffle => self.letters.shuffle(&mut rand::thread_rng()),
            Msg::Backspace => {
                self.current_word.pop();
            }
            Msg::Keyboard => {
                let window = web_sys::window();
                let document = window
                    .unwrap()
                    .document()
                    .expect("window should have a document");
                let _ = document
                    .query_selector("#hiddeninput")
                    .unwrap()
                    .unwrap()
                    .dyn_ref::<HtmlElement>()
                    .unwrap()
                    .focus();
            }
            Msg::Submit => {
                if !self.wordlist.words.contains(&self.current_word) {
                    self.message = Some(error(&self.wordlist, self.current_word.as_str()));
                    let link = self.link.clone();
                    Timeout::new(1000, move || link.send_message(Msg::ClearMessage)).forget();
                    self.current_word.clear();
                } else if self.found_words.contains(&self.current_word) {
                    self.message = Some("Already found".into());
                    self.current_word.clear();
                } else {
                    self.found_words
                        .push(std::mem::take(&mut self.current_word))
                }
                self.local_storage
                    .set_item(
                        &key(self.center, &self.letters),
                        &self.found_words.join("\n"),
                    )
                    .unwrap();
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
        fn keyboard_callback(msg: keyboard::Msg) -> Msg {
            match msg {
                keyboard::Msg::Char(c) => Msg::PushLetter(c),
                keyboard::Msg::Shuffle => Msg::Shuffle
            }
        }
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
                <div class="sb-message-box error-message">
                    <div class="sb-message">{message}</div>
                </div>
            },
            None => html! { <div class="sb-message-box" /> },
        };

        let current_word = self
            .current_word
            .chars()
            .map(|ch| {
                if ch == self.center {
                    html! { <span class="sb-input-bright"> { ch } </span> }
                } else if !self.letters.contains(&ch) {
                    html! { <span class="sb-input-extra"> { ch } </span> }
                } else {
                    html! { <span> { ch } </span> }
                }
            })
            .collect::<Html>();
        //let current_word = self.current_word.clone();
        let words = self
            .found_words
            .iter()
            .map(|word| html! { <li>{word}</li> })
            .collect::<Html>();
        let valid_words = &self.wordlist.words;
        let dots = (0..valid_words.len())
            .into_iter()
            .map(|i| {
                if i <= self.found_words.len() {
                    html! { <span class="sb-progress-dot completed" /> }
                } else {
                    html! { <span class="sb-progress-dot" /> }
                }
            })
            .collect::<Html>();
        let offset = (100.0 / valid_words.len() as f64) * self.found_words.len() as f64;
        let progress = html! {
            <span role="presentation">
                <div class="sb-progress" title="Click to see today’s ranks">
                  <h4 class="sb-progress-rank">{ "Amazing Human/Genius" }</h4>
                  <div class="sb-progress-bar">
                  <div class="sb-progress-line">
                    <div class="sb-progress-dots">
                        {dots}
                    </div>
            </div>
            <div class="sb-progress-marker" style={ format!("left: {}%", offset)}><span class="sb-progress-value"> { self.found_words.len() }</span></div></div></div>
            </span>
        };
        let hidden = if self.wordlist_visible { "wordlist-drawer" } else { "wordlist-drawer hidden" };
        let showhide_text = if self.wordlist_visible { "Hide" } else { "Show" };
        let showhide = html! { <button onclick={self.link.callback(|_|Msg::ToggleWords)}>{ showhide_text }</button> };
        let wordlist = html! {
                    <div class="wordlist-box">
                        <div class="wordlist-heading">
                            <div class="wordlist-summary">{ format!("You have found {} words", self.found_words.len()) }</div>
                            {showhide}
                        </div>
                        <div class={hidden}>
                            <div class="wordlist-window">
                                <div class="wordlist-pag">
                                    <div class="sb-wordlist-scroll-anchor" style="left: 0%;"></div>
                                    <ul class="sb-wordlist-items-pag single">
                                        {words}
                                    </ul>
                                </div>
                                <div class="sb-kebob"></div>
                            </div>
                        </div>
                    </div>
        };
        /*let inner = html! {
            <div class="sb-content-box">
                <div class="sb-status-box">
                    <div class="sb-progress-box">{ progress }</div>
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
                <div class="sb-controls-box">
                    <div class="sb-controls">
                        {{ message }}
                        <div class="sb-hive-input">
                            <span class="sb-hive-input-content non-empty" style="font-size: 1em;">
                                <span class="">{{ current_word }}</span>
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
                            <div onclick=self.link.callback(|_|Msg::Keyboard) class="hive-action hive-action__keyboard sb-touch-button">{"Keyboard"}</div>
                        </div>
                    </div>
                </div>
                <Keyboard disabled={ HashSet::new() } purple={ None } ontype= { self.link.callback(keyboard_callback) } />
            </div>
        };
        wrap(inner)*/
        html! {
            <div class="container">
                { wordlist }
                <div class="sb-hive-input">
                    <span class="sb-hive-input-content non-empty" style="font-size: 1em;">
                        <span class="">{{ current_word }}</span>
                    </span>
                </div>
                <div class="sb-hive hive-container">
                    <div class="hive">
                        {{ center }}
                        {{ letters }}
                    </div>
                </div>
                <div class="hive-actions">
                    <div onclick=self.link.callback(|_|Msg::Submit) class="hive-action hive-action__submit sb-touch-button">{ "Enter" }</div>
                    <div onclick=self.link.callback(|_|Msg::Backspace) class="hive-action hive-action__delete sb-touch-button">{"Delete"}</div>
                </div>
                <div class="keyboard-footer">
                    <Keyboard purple={self.purple()} grid={self.grid()} ontype={ self.link.callback(keyboard_callback) } />
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<SpellingBee>();
}
