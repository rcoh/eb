use std::collections::HashSet;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

pub enum Msg {
    Char(char),
    Shuffle
}

const SHUFFLE: char = '🌀';

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub disabled: HashSet<char>,
    pub purple: Option<char>,
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
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        _props != self.props
    }

    fn view(&self) -> Html {
        let rows = &["qwertyuiop", "asdfghjkl", "zxcvbnm🌀"];
        let keyboard = rows.iter().enumerate().flat_map(
            |(row_idx, row)|row.chars().enumerate().map(
                move |(col_idx, key)| make_hexagon(key, row_idx, col_idx, &self.link))
        ).collect::<Html>();
        html! {
            <div class="keyboard-container">
                { keyboard }
            </div>
        }
    }
}

fn compute_transform(row: usize, col: usize) -> String {
    let y: i32 = 75 * row as i32;
    let x = (col * 100) + row * 50;
    format!("transform: translateX({x}%) translateY({y}%)", x=x,y=y)
}

fn make_hexagon(letter: char, row: usize, col: usize, link: &ComponentLink<Keyboard>) -> Html {
    // points
    // 0,30 51.96,0 120,30
    html! {
        <svg class="keyboard-cell" onclick={link.callback(move |_|letter)} style={ compute_transform(row, col) } viewBox="0 0 103.92304845413263 120">
            <polygon class="enabled" points="0,30 0,90 51.96152422706631,120 103.92304845413263,90 103.92304845413263,30 51.96152422706631,0" stroke="white" stroke-width="7.5">
            </polygon>
            <text class="keyboard-letter" x="50%" y="50%" dy="0.35em">{ letter }</text>
        </svg>
    }
}
