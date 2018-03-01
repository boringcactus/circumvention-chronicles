#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate find_folder;

use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::OpenGL;
use piston_window::texture::UpdateTexture;

mod support;

// Based on conrod examples
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
    let mut image_map = conrod::image::Map::new();
    let mut app = support::DemoApp::new();

    while let Some(event) = window.next() {
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            support::gui(&mut ui, &ids, &mut app);
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
    }
}
