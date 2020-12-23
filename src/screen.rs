extern crate minifb;

use minifb::{ScaleMode, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Screen {
    pixels: [u32; WIDTH * HEIGHT],
    window: Window,
}

impl Screen {
    pub fn new() -> Self {
        let mut window = Window::new(
            "CHIPO",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X8,
                scale_mode: ScaleMode::UpperLeft,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Screen {
            window,
            pixels: [0; WIDTH * HEIGHT],
        }
    }

    pub fn init(&mut self) {}

    pub fn draw(&mut self) {
        self.window
            .update_with_buffer(&self.pixels, WIDTH, HEIGHT)
            .unwrap();
    }

    pub fn clear(&mut self) {}
}
