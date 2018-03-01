extern crate reqwest;

use open;
use rand;
use rand::Rng;

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

pub enum CacheState {
    Waiting(String),
    Completed
}

fn alnum() -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(5, 20);
    let text = rng.gen_ascii_chars().take(length).collect();
    text
}

fn path(picture: bool) -> String {
    let mut rng = rand::thread_rng();
    let path_segments = rng.gen_range(1, 5);
    let extension = if picture {
        rng.choose(&["jpg", "jpeg", "png", "gif"]).unwrap()
    } else {
        rng.choose(&["html", "txt", "md", "pdf", "tex", "exe", "deb", "tgz"]).unwrap()
    };
    let path = (0..path_segments)
        .map(|_| format!("/{}", alnum()))
        .collect::<Vec<_>>()
        .join("") + "." + extension;
    path
}

fn garbage() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(1000, 10000);
    let mut data = vec![0u8; length];
    rng.fill_bytes(&mut data);
    data
}

fn build_cache(target_path: &str) -> Vec<u8> {
    let mut result = vec![];
    // prefix some nonsense
    result.extend(garbage());
    // pick a number of other files to put before
    let mut rng = rand::thread_rng();
    let prefix_files = rng.gen_range(1, 5);
    for _ in 0..prefix_files {
        write!(result, "http://localhost:{}{}", PORT, path(false)).unwrap();
        result.extend(garbage());
    }
    // put the file itself
    write!(result, "http://localhost:{}{}", PORT, target_path).unwrap();
    result.extend(garbage());
    // suffix some things too
    let suffix_files = rng.gen_range(1, 5);
    for _ in 0..suffix_files {
        write!(result, "http://localhost:{}{}", PORT, path(false)).unwrap();
        result.extend(garbage());
    }
    result
}

impl Default for CacheState {
    fn default() -> CacheState {
        let path = path(true);
        CacheState::Waiting(path)
    }
}

pub enum Level {
    TitleScreen,
    Tutorial(bool),
    Offline(OfflineState),
    Cache(CacheState),
    // Wikipedia,
    // Tor,
    // SSHDynamic,
    // Freenet,
    // WebArchive,
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
            &Level::Cache(_) => "Circumvention Chronicles LEVEL 2".to_string(),
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
            &Level::Cache(CacheState::Waiting(_)) => "So you've been downloading porn when you have Internet and viewing it \
                when you don't, and that's been working out OK, but now once you close a tab you lose that material. \
                Poking through the browser history sometimes works, but sometimes the browser doesn't still have it downloaded. \
                What if you just asked the browser which things it still had?".to_string(),
            &Level::Cache(CacheState::Completed) => "Very well done! Years later you'll think this is so ridiculous it would stink \
                to leave it out of this very game, but until then you can just move on with your life.".to_string(),
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
            &Level::Cache(CacheState::Waiting(_)) => "Open Browser\nStorage",
            &Level::Cache(CacheState::Completed) => "Next Puzzle",
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
            &Level::Offline(OfflineState::Completed) => Some(Level::Cache(CacheState::default())),
            &Level::Cache(CacheState::Waiting(_)) => {
                launch_web();
                None
            },
            &Level::Cache(CacheState::Completed) => Some(Level::Credits),
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
            &Level::Cache(CacheState::Waiting(_)) => "What type of file are you looking for?".to_string(),
            &Level::Cache(CacheState::Completed) => cont,
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
            (&Level::Cache(CacheState::Waiting(_)), &Method::Get, "/") => (None, read_file("assets/cache/index.html")),
            (&Level::Cache(CacheState::Waiting(ref path)), &Method::Get, "/cache") => (None, Some(build_cache(path).into())),
            (&Level::Cache(CacheState::Waiting(ref good_path)), &Method::Get, real_path) => {
                if good_path == real_path {
                    (Some(Level::Cache(CacheState::Completed)), read_file("assets/cache/congrats.html"))
                } else {
                    (None, None)
                }
            },
            _ => (None, None)
        }
    }
}
