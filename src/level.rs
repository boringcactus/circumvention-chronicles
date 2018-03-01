use open;

use hyper::{Method, Body};
use std::fs::File;
use std::io::prelude::*;
use super::http::PORT;

// The Option isn't success/failure, it's 404/no-404
// and since we always call it with a str, there's no point taking a Path
fn read_file(path: &str) -> Option<Body> {
    if let Ok(mut file) = File::open(path) {
        let mut contents: Vec<u8> = vec![];
        file.read_to_end(&mut contents).expect("Could not read file from assets");
        Some(contents.into())
    } else {
        None
    }
}

pub enum Level {
    TitleScreen,
    Tutorial(bool),
    Credits,
}

impl Default for Level {
    fn default() -> Level {
        Level::TitleScreen
    }
}

fn launch_web() {
    launch_web_at_path("/");
}

fn launch_web_at_path(path: &str) {
    open::that(format!("http://localhost:{}{}", PORT, path)).unwrap();
}

impl Level {
    pub fn description(&self) -> String {
        match self {
            &Level::TitleScreen => "Circumvention Chronicles is a Web-based puzzle game and filter circumvention tutorial. \
                You'll be presented with all sorts of scenarios where your Internet access is constrained in some way. \
                Solve puzzles in your Web browser to find porn anyways using some of the same methods I used in my misspent adolescence.".to_string(),
            &Level::Tutorial(false) => "We'll start with a very basic puzzle to make sure everything works right. \
                Hit the \"Open Browser\" button below and follow the instructions in the page that opens.".to_string(),
            &Level::Tutorial(true) => "Well done. Now the real fun begins.".to_string(),
            &Level::Credits => "You have finished the game. Thank you for playing!\n\
                Circumvention Chronicles is a game made by Melody (@boring_cactus)".to_string(),
        }
    }

    pub fn button_label(&self) -> &str {
        match self {
            &Level::TitleScreen => "Continue",
            &Level::Tutorial(false) => "Open Browser",
            &Level::Tutorial(true) => "Continue",
            &Level::Credits => "Open itch.io page",
        }
    }

    pub fn handle_button_press(&self) -> Option<Self> {
        match self {
            &Level::TitleScreen => Some(Level::Tutorial(false)),
            &Level::Tutorial(false) => {
                launch_web();
                None
            },
            &Level::Tutorial(true) => Some(Level::Credits),
            &Level::Credits => {
                open::that("https://boringcactus.itch.io/circumvention-chronicles").unwrap();
                None
            },
        }
    }

    pub fn hint(&self) -> String {
        let cont = "Click the \"Continue\" button above".to_string();
        match self {
            &Level::TitleScreen => cont,
            &Level::Tutorial(false) => format!("Open http://localhost:{} with the button above and click the \"Continue\" button on that page", PORT),
            &Level::Tutorial(true) => cont,
            &Level::Credits => "*notices your hint* OwO what's this?".to_string(),
        }
    }

    pub fn handle_request(&self, method: &Method, path: &str) -> (Option<Self>, Option<Body>) {
        match (self, method, path) {
            (_, &Method::Get, "/style.css") => (None, read_file("assets/style.css")),
            (&Level::Tutorial(false), &Method::Get, "/") => (None, read_file("assets/demo/index.html")),
            (&Level::Tutorial(false), &Method::Post, "/complete") => (Some(Level::Tutorial(true)), read_file("assets/demo/congrats.html")),
            _ => (None, None)
        }
    }
}
