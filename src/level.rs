use open;

use hyper::Method;

#[derive(Clone)]
pub enum Level {
    Tutorial,
    Demo(u8),
}

impl Level {
    pub fn description(&self) -> String {
        match self {
            &Level::Tutorial => "Circumvention Chronicles is a Web-based historical reenactment puzzle game where the \
                history you're reenacting is mine.".to_string(),
            &Level::Demo(ref i) => format!("You have submitted from the page {} times", i),
        }
    }

    pub fn button_label(&self) -> &str {
        match self {
            &Level::Tutorial => "Continue",
            &Level::Demo(_) => "Open Browser",
        }
    }

    pub fn handle_button_press(&self) -> Self {
        match self {
            &Level::Tutorial => Level::Demo(0),
            &Level::Demo(_) => {
                open::that("http://localhost:9866").unwrap();
                Level::Demo(0)
            }
        }
    }

    pub fn hint(&self) -> &str {
        match self {
            &Level::Tutorial => "Just click the button",
            &Level::Demo(_) => "Click the button again, then reload the page a bunch",
        }
    }

    pub fn handle_request(&self, method: &Method, path: &str) -> (Option<Self>, Option<String>) {
        match (self, method, path) {
            (&Level::Demo(_), &Method::Get, "/") => (None, Some("<html><form method='POST'><input type='submit'></form></html>".to_string())),
            (&Level::Demo(ref i), &Method::Post, "/") => (Some(Level::Demo(i + 1)), Some("cool thanks".to_string())),
            _ => (None, None)
        }
    }
}
