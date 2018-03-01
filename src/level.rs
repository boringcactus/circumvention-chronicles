extern crate reqwest;

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

pub enum OfflineState {
    Waiting,
    Loaded,
    Busted,
    Completed
}

pub enum Level {
    TitleScreen,
    Tutorial(bool),
    Offline(OfflineState),
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

// Used for Level 1. Makes some dubious assumptions.
fn is_online() -> bool {
    let response = reqwest::get("http://example.com");
    match response {
        Ok(_) => true,
        Err(_) => {
            // This winds up being an IO error if offline.
            // There's definitely a way to detect only that. I am lazy.
            false
        }
    }
}

impl Level {
    pub fn title(&self) -> String {
        match self {
            &Level::TitleScreen => "Circumvention Chronicles".to_string(),
            &Level::Tutorial(_) => "Circumvention Chronicles TUTORIAL".to_string(),
            &Level::Offline(_) => "Circumvention Chronicles LEVEL 1".to_string(),
            &Level::Credits => "Circumvention Chronicles CREDITS".to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            &Level::TitleScreen => "Circumvention Chronicles is a Web-based puzzle game and filter circumvention tutorial. \
                You'll be presented with all sorts of scenarios where your Internet access is constrained in some way. \
                Solve puzzles in your Web browser to find porn anyways using some of the same methods I used in my misspent adolescence.".to_string(),
            &Level::Tutorial(false) => "We'll start with a very basic puzzle to make sure everything works right. \
                Hit the \"Open Browser\" button below and follow the instructions in the page that opens.".to_string(),
            &Level::Tutorial(true) => "Well done. Now the real fun begins.".to_string(),
            &Level::Offline(OfflineState::Busted) => "Oh no! You had porn open in a place where people could see it! \
                You are now having an unfortunate and awkward conversation with either your teachers or your parents.".to_string(),
            &Level::Offline(OfflineState::Completed) => "Nice job! Eventually you'll install a Firefox extension that minimizes your \
                window full of porn tabs to the system tray so that you can't even accidentally open it in public.".to_string(),
            &Level::Offline(OfflineState::Waiting) | &Level::Offline(OfflineState::Loaded) => "You have a laptop. \
                At school, you have WiFi, but no privacy. At home, you have privacy, but no WiFi. \
                (This restriction of privacy and Internet connectivity being mutually exclusive is simulated for this puzzle.) \
                You've heard some interesting things about this concept of \"pornography\" and you're curious.".to_string(),
            &Level::Credits => "You have finished the game. Thank you for playing!\n\
                Circumvention Chronicles is a game made by Melody (@boring_cactus)".to_string(),
        }
    }

    pub fn button_label(&self) -> &str {
        match self {
            &Level::TitleScreen => "Continue",
            &Level::Tutorial(false) => "Open Browser",
            &Level::Tutorial(true) => "Next Puzzle",
            &Level::Offline(OfflineState::Waiting) | &Level::Offline(OfflineState::Loaded) => "Go Online",
            &Level::Offline(OfflineState::Busted) => "Retry Puzzle",
            &Level::Offline(OfflineState::Completed) => "Next Puzzle",
            &Level::Credits => "Open\nitch.io page",
        }
    }

    pub fn handle_button_press(&self) -> Option<Self> {
        match self {
            &Level::TitleScreen => Some(Level::Tutorial(false)),
            &Level::Tutorial(false) => {
                launch_web();
                None
            },
            &Level::Tutorial(true) => Some(Level::Offline(OfflineState::Waiting)),
            &Level::Offline(OfflineState::Waiting) | &Level::Offline(OfflineState::Loaded) => {
                launch_web();
                None
            },
            &Level::Offline(OfflineState::Busted) => Some(Level::Offline(OfflineState::Waiting)),
            &Level::Offline(OfflineState::Completed) => Some(Level::Credits),
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
            &Level::Offline(OfflineState::Waiting) => "How can you load the resources on a page without looking directly at it immediately?".to_string(),
            &Level::Offline(OfflineState::Loaded) => "Now that the porn is loaded, how can you gain privacy so you can look at it?".to_string(),
            &Level::Offline(OfflineState::Busted) => "This may require interacting with your computer in ways that aren't part of the game.".to_string(),
            &Level::Offline(OfflineState::Completed) => cont,
            &Level::Credits => "*notices your hint* OwO what's this?".to_string(),
        }
    }

    pub fn handle_request(&self, method: &Method, path: &str) -> (Option<Self>, Option<Body>) {
        match (self, method, path) {
            (_, &Method::Get, "/style.css") => (None, read_file("assets/style.css")),
            (&Level::Tutorial(false), &Method::Get, "/") => (None, read_file("assets/demo/index.html")),
            (&Level::Tutorial(false), &Method::Post, "/complete") => (Some(Level::Tutorial(true)), read_file("assets/demo/congrats.html")),
            (&Level::Offline(OfflineState::Completed), _, _) => (None, read_file("assets/offline/congrats.html")),
            (&Level::Offline(_), &Method::Get, "/") => {
                if is_online() {
                    (None, read_file("assets/offline/index.html"))
                } else {
                    (None, read_file("assets/offline/offline.html"))
                }
            },
            (&Level::Offline(_), &Method::Get, "/lewd.html") => {
                if is_online() {
                    (Some(Level::Offline(OfflineState::Loaded)), read_file("assets/offline/lewd.html"))
                } else {
                    (None, read_file("assets/offline/offline.html"))
                }
            },
            (&Level::Offline(_), &Method::Get, "/saw-lewd") => {
                if is_online() {
                    (Some(Level::Offline(OfflineState::Busted)), read_file("assets/offline/busted.html"))
                } else {
                    (Some(Level::Offline(OfflineState::Completed)), read_file("assets/offline/congrats.html"))
                }
            },
            _ => (None, None)
        }
    }
}
