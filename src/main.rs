use rand::seq::SliceRandom;
use wasm_bindgen::{prelude::Closure, JsCast};

use std::collections::HashSet;
use yew::{classes, html, Component, Context, Html, KeyboardEvent};

const WORDS: &str = include_str!("../word-list.txt");
const ALLOWED_KEYS: [char; 29] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];
const EMPTY: char = '\u{00a0}';

fn parse_words(words: &str) -> Vec<Vec<char>> {
    let mut word_list = Vec::new();

    for word in words.lines() {
        if word.chars().count() == 5 {
            word_list.push(word.chars().collect());
        }
    }

    word_list
}

enum Msg {
    KeyPress(char),
    Backspace,
    Guess,
    Noop,
}

struct Model {
    word_list: Vec<Vec<char>>,
    word: Vec<char>,

    is_guessing: bool,
    is_winner: bool,
    message: String,

    present_characters: HashSet<char>,
    correct_characters: HashSet<(char, usize)>,
    absent_characters: HashSet<char>,

    guesses: [Vec<char>; 6],
    current_guess: usize,

    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CharacterState {
    Unknown,
    NotInWord,
    InWord,
    InCorrectPosition,
}

impl Model {
    fn map_character_state(&self, character: char, index: usize) -> Option<&'static str> {
        if self.correct_characters.contains(&(character, index)) {
            Some("correct")
        } else if self.absent_characters.contains(&character) {
            Some("absent")
        } else if self.present_characters.contains(&character) {
            Some("present")
        } else {
            None
        }
    }

    fn map_keyboard_state(&self, character: char) -> Option<&'static str> {
        if self.correct_characters.iter().any(|(c, _)| *c == character) {
            Some("correct")
        } else if self.absent_characters.contains(&character) {
            Some("absent")
        } else if self.present_characters.contains(&character) {
            Some("present")
        } else {
            None
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let word_list = parse_words(WORDS);

        let word = word_list.choose(&mut rand::thread_rng()).unwrap().clone();

        Self {
            word,
            word_list,
            is_guessing: true,
            is_winner: false,
            message: EMPTY.to_string(),
            present_characters: HashSet::new(),
            correct_characters: HashSet::new(),
            absent_characters: HashSet::new(),
            guesses: [
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
                Vec::with_capacity(5),
            ],
            current_guess: 0,
            keyboard_listener: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let window: web_sys::Window = web_sys::window().expect("window not available");

        let cb = ctx.link().callback(|e: KeyboardEvent| {
            if e.key().chars().count() == 1 {
                let key = e.key().to_uppercase().chars().next().unwrap();
                if ALLOWED_KEYS.contains(&key) {
                    Msg::KeyPress(key)
                } else {
                    Msg::Noop
                }
            } else if e.key() == "Backspace" {
                Msg::Backspace
            } else if e.key() == "Enter" {
                Msg::Guess
            } else {
                Msg::Noop
            }
        });

        // Create a Closure from a Box<dyn Fn> - this has to be 'static
        let listener =
            Closure::<dyn Fn(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| cb.emit(e)));

        window
            .add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
            .unwrap();
        self.keyboard_listener = Some(listener);
    }

    fn destroy(&mut self, _: &Context<Self>) {
        // remove the keyboard listener
        if let Some(listener) = self.keyboard_listener.take() {
            let window: web_sys::Window = web_sys::window().expect("window not available");
            window
                .remove_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
                .unwrap();
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyPress(c) => {
                if !self.is_guessing || self.guesses[self.current_guess].len() >= 5 {
                    return false;
                }

                self.message = EMPTY.to_string();
                self.guesses[self.current_guess].push(c);

                true
            }
            Msg::Backspace => {
                if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
                    return false;
                }

                self.message = EMPTY.to_string();
                self.guesses[self.current_guess].pop();

                true
            }
            Msg::Guess => {
                if self.guesses[self.current_guess].len() != 5 {
                    self.message = String::from("Liian vähän kirjaimia!");
                    return true;
                }

                if !self.word_list.contains(&self.guesses[self.current_guess]) {
                    self.message = String::from("Ei sanalistalla");
                    return true;
                }

                self.is_winner = self.guesses[self.current_guess] == self.word;

                for (index, character) in self.guesses[self.current_guess].iter().enumerate() {
                    if self.word[index] == *character {
                        self.correct_characters.insert((*character, index));
                    }

                    if self.word.contains(character) {
                        self.present_characters.insert(*character);
                    } else {
                        self.absent_characters.insert(*character);
                    }
                }

                if self.is_winner {
                    self.is_guessing = false;
                    self.message = String::from("Löysit sanan! 🥳");
                } else if self.current_guess == 5 {
                    self.is_guessing = false;
                    self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
                } else {
                    self.message = EMPTY.to_string();
                    self.current_guess += 1;
                }

                true
            }
            Msg::Noop => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let keyboard = vec![
            vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Å'],
            vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'],
            vec!['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
        ];

        html! {
            <div class="game">
                <header>
                    <div class="title">{ "Sanuli" }</div>
                </header>

                <div class="board-container">
                    <div class="board">
                        { self.guesses.iter().enumerate().map(|(g_index, guess)| html! {
                            <div class="row">
                                { (0..5).map(|c_index| html! {
                                    <div class={classes!(
                                        "tile",
                                        guess.get(c_index).and_then(|c| self.map_character_state(*c, c_index)),
                                        if self.is_guessing && g_index == self.current_guess { Some("current") } else { None }
                                    )}>
                                        { guess.get(c_index).unwrap_or(&' ') }
                                    </div>
                                }).collect::<Html>() }
                            </div>
                        }).collect::<Html>() }
                    </div>
                </div>

                <div class="keyboard">
                    <div class="message">{ &self.message }</div>
                    <div class="keyboard-row">
                        {
                            keyboard[0].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <div class="spacer" />
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        {
                            keyboard[1].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="keyboard-row">
                        <div class="spacer" />
                        <div class="spacer" />
                        {
                            keyboard[2].iter().cloned().map(|key| html! {
                                <button class={classes!("keyboard-button", self.map_keyboard_state(key))}
                                    onclick={link.callback(move |_| Msg::KeyPress(key))}>{ key }</button>
                            }).collect::<Html>()
                        }
                        <button class={classes!("keyboard-button")}
                            onclick={link.callback(move |_| Msg::Backspace)}>{ "⌫" }</button>
                        <button class={classes!("keyboard-button")} onclick={link.callback(|_| Msg::Guess)}>{ "ARVAA" }</button>
                        <div class="spacer" />
                        <div class="spacer" />
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
    // let app = yew::start_app_as_body::<Model>();
}
