extern crate rand;

use conrod;
use std;
use std::sync::{Mutex, Arc};

use hyper::{Method, StatusCode, Response};

use super::http;
use super::level::Level;

// Stolen from conrod examples

pub struct GameState {
    pub approved: bool,
    pub current_level: Level,
    showing_hint: bool,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            approved: false,
            current_level: Level::default(),
            showing_hint: false,
        }
    }

    pub fn description(&self) -> String {
        match self.approved {
            false => format!("Circumvention Chronicles is only for adults. It does not contain porn, but it is \
                about porn. Additionally, it relies on a locally bound Web server on port {} \
                that may trigger firewall warnings. If you are 18 or older and you are comfortable \
                with this, press \"Begin\". If not, press Esc or close this window.", http::PORT),
            true => self.current_level.description()
        }
    }

    pub fn button_label(&self) -> &str {
        match self.approved {
            false => "Begin",
            true => self.current_level.button_label()
        }
    }

    pub fn handle_button_press(&mut self) {
        if !self.approved {
            self.approved = true;
        } else {
            if let Some(new_level) = self.current_level.handle_button_press() {
                self.current_level = new_level;
            }
        }
    }

    pub fn toggle_hint(&mut self) {
        self.showing_hint = !self.showing_hint;
    }

    pub fn hint(&self) -> String {
        match self.approved {
            false => String::new(),
            true => self.current_level.hint()
        }
    }

    pub fn handle_request(&mut self, method: &Method, path: &str) -> Response {
        match self.approved {
            false => Response::new().with_status(StatusCode::Unauthorized),
            true => {
                let (new_level, response) = self.current_level.handle_request(method, path);
                if let Some(new_level) = new_level {
                    self.current_level = new_level;
                }
                match response {
                    Some(result) => Response::new().with_body(result),
                    None => Response::new().with_status(StatusCode::NotFound)
                }
            }
        }
    }
}

pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {

        // The scrollable canvas.
        canvas,

        // The title and introduction widgets.
        title,
        introduction,

        // Button, XyPad, Toggle.
        action_button,
        hint_button,
        hint_text,

        // Scrollbar
        canvas_scrollbar,

    }
}


/// Instantiate a GUI demonstrating every widget available in conrod.
pub fn gui(ui: &mut conrod::UiCell, ids: &Ids, state_mutex: &Arc<Mutex<GameState>>) {
    use conrod::{widget, Labelable, Positionable, Sizeable, Widget};

    let mut state = state_mutex.lock().unwrap();

    const MARGIN: conrod::Scalar = 30.0;
    const TITLE_SIZE: conrod::FontSize = 42;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    const TITLE: &'static str = "Circumvention Chronicles";
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);


    ////////////////
    ///// TEXT /////
    ////////////////


    // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
    // introduction to the example.
    widget::Text::new(TITLE).font_size(TITLE_SIZE).mid_top_of(ids.canvas).set(ids.title, ui);

    let desc = state.description();
    widget::Text::new(&desc)
        .padded_w_of(ids.canvas, MARGIN)
        .down(2.0 * MARGIN)
        .align_middle_x_of(ids.canvas)
        .left_justify()
        .line_spacing(5.0)
        .set(ids.introduction, ui);


    /////////////////////////////////
    ///// Button, XYPad, Toggle /////
    /////////////////////////////////


    let side = 130.0;

    let label = state.button_label().to_string();
    for _press in widget::Button::new()
        .label(&label)
        .mid_left_with_margin_on(ids.canvas, MARGIN)
        .down_from(ids.introduction, 2.0 * MARGIN)
        .w_h(side, side)
        .set(ids.action_button, ui)
    {
        state.handle_button_press();
    }

    if state.approved {
        let hint_label = match state.showing_hint {
            false => "Show Hint",
            true => "Hide Hint"
        };
        for _press in widget::Button::new()
            .label(hint_label)
            .down_from(ids.action_button, 1.5 * MARGIN)
            .set(ids.hint_button, ui)
        {
            state.toggle_hint();
        }

        if state.showing_hint {
            let hint = state.hint().to_string();
            widget::Text::new(&hint)
                .right_from(ids.hint_button, 1.5 * MARGIN)
                .align_middle_y_of(ids.hint_button)
                .set(ids.hint_text, ui);
        }
    }

    /////////////////////
    ///// Scrollbar /////
    /////////////////////


    widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.canvas_scrollbar, ui);
}
