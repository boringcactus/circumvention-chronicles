#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate find_folder;
extern crate futures;
extern crate hyper;
extern crate open;
extern crate rand;

use std::thread;
use std::sync::{Mutex, Arc};

use futures::Future;
use futures::sync::oneshot;

use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use piston_window::{G2d, G2dTexture, TextureSettings};
use piston_window::OpenGL;
use piston_window::texture::UpdateTexture;

mod support;
mod level;
mod http;

// Based on conrod and simple-server examples
fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut window: PistonWindow = WindowSettings::new("Circumvention Chronicles", [WIDTH, HEIGHT])
        .opengl(OpenGL::V3_2)
        .samples(4)
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();

    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
        .theme(support::theme())
        .build();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);
        let buffer_len = WIDTH as usize * HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;
        let texture = G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap();
        (cache, texture)
    };

    let ids = support::Ids::new(ui.widget_id_generator());
    let image_map = conrod::image::Map::new();
    let state_mutex = Arc::new(Mutex::new(support::GameState::new()));

    let (server_kill_sender, server_kill_receiver) = oneshot::channel();
    let server_kill_receiver = server_kill_receiver.shared();

    let mut server_handle = None;

    while let Some(event) = window.next() {
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            support::gui(&mut ui, &ids, &state_mutex);
        });

        window.draw_2d(&event, |context, graphics| {
            if let Some(primitives) = ui.draw_if_changed() {
                let cache_queued_glyphs = |graphics: &mut G2d,
                                           cache: &mut G2dTexture,
                                           rect: conrod::text::rt::Rect<u32>,
                                           data: &[u8]|
                {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    let encoder = &mut graphics.encoder;
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                        .expect("failed to update texture");
                };
                fn texture_from_image<T>(img: &T) -> &T { img }

                conrod::backend::piston::draw::primitives(primitives, context, graphics,
                    &mut text_texture_cache, &mut glyph_cache, &image_map, cache_queued_glyphs,
                    texture_from_image);
            }
        });

        if let None = server_handle {
            let state = state_mutex.lock().unwrap();
            if state.approved {
                let server_state_mutex = Arc::clone(&state_mutex);
                let recv = server_kill_receiver.clone();
                server_handle = Some(thread::spawn(move || {
                    http::GameServer::run(server_state_mutex, recv);
                }));
            }
        }
    }

    drop(window);

    server_kill_sender.send(()).unwrap();
    if let Some(server_handle) = server_handle {
        server_handle.join().unwrap();
    }
}
