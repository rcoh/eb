use std::collections::HashSet;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::web_sys::console::log;

pub enum Msg {
    Char(char),
    Shuffle
}

const SHUFFLE: char = '↺';

#[derive(Clone, Properties, PartialEq, Debug)]
pub struct Props {
    pub purple: Option<char>,
    pub grid: HashSet<char>,
    pub ontype: Callback<Msg>,
}

pub struct Keyboard {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for Keyboard {
    type Message = char;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Keyboard { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            SHUFFLE => self.props.ontype.emit(Msg::Shuffle),
            letter => self.props.ontype.emit(Msg::Char(letter))
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        ConsoleService::log(&format!("props[change]: {:?}", _props));
        let changed = _props != self.props;
        self.props = _props;
        changed
    }

    fn view(&self) -> Html {
        ConsoleService::log(&format!("props: {:?}", &self.props));
        let rows = &["qwertyuiop".to_string(), "asdfghjkl".to_string(), format!("zxcvbnm{}", SHUFFLE)];
        let keyboard = rows.iter().enumerate().flat_map(
            |(row_idx, row)|row.chars().enumerate().map(
                move |(col_idx, key)| make_hexagon(key, row_idx, col_idx, &self.link, self.letter_status(key)))
        ).collect::<Html>();
        html! {
            <div class="keyboard-container">
                { keyboard }
            </div>
        }
    }
}

impl Keyboard {
    fn letter_status(&self, letter: char) -> Status {
        if letter == SHUFFLE {
            return Status::Normal
        }
        if self.props.grid.contains(&letter) {
            return Status::InGrid
        }
        match self.props.purple {
            None => Status::Normal,
            Some(purple) if letter == purple => Status::Purple,
            Some(_disabled) => Status::Disabled
        }
    }
}

fn compute_transform(row: usize, col: usize) -> String {
    let y: i32 = 75 * row as i32;
    let x = (col * 100) + row * 50;
    format!("transform: translateX({x}%) translateY({y}%)", x=x,y=y)
}

enum Status {
    InGrid,
    Normal,
    Disabled,
    Purple
}

impl Status {
    fn class(&self) -> &'static str {
        match self {
            Status::InGrid => "grid-letter",
            Status::Normal => "normal-letter",
            Status::Disabled => "disabled-letter",
            Status::Purple => "purple-letter"
        }
    }
}


fn make_hexagon(letter: char, row: usize, col: usize, link: &ComponentLink<Keyboard>, status: Status) -> Html {
    // points
    // 0,30 51.96,0 120,30
    let class = format!("keyboard-letter {}-text", status.class());
    html! {
        <svg class="keyboard-cell" onclick={link.callback(move |_|letter)} style={ compute_transform(row, col) } viewBox="0 0 103.92304845413263 120">
            <polygon class={status.class()} points="0,30 0,90 51.96152422706631,120 103.92304845413263,90 103.92304845413263,30 51.96152422706631,0" stroke="white" stroke-width="7.5">
            </polygon>
            <text class={class} x="50%" y="50%" dy="0.35em">{ letter }</text>
        </svg>
    }
}
