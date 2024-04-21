use std::{
    fs::File,
    io::{self, Read},
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    error::EventLoopError,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::Key,
    window::{Window, WindowBuilder},
};

use super::cpu::{Cpu, PROGRAM_START, SCREEN_HEIGHT, SCREEN_WIDTH};

const SCALING: usize = 8;

pub struct Emulator {
    event_loop: EventLoop<()>,
    window: Window,
    screen_renderer: Pixels,
    cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = {
            let size = LogicalSize::new((SCREEN_WIDTH * SCALING) as f64, (SCREEN_HEIGHT * SCALING) as f64);
            WindowBuilder::new()
                .with_title("Chipo Emulator")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let screen_renderer = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
        };
        event_loop.set_control_flow(ControlFlow::Poll);

        Self {
            window,
            event_loop,
            screen_renderer,
            cpu: Cpu::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), io::Error> {
        let program_data = Emulator::load_rom_file(path)?;
        for i in 0..program_data.len() {
            self.cpu.write(PROGRAM_START + (i as u16), program_data[i]);
        }
        Ok(())
    }

    fn load_rom_file(path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        let bytes_read = file.read_to_end(&mut buffer)?;

        println!("Loaded '{}' ({} bytes read)", path, bytes_read);

        Ok(buffer)
    }

    pub fn run(mut self) -> Result<(), EventLoopError> {
        self.event_loop
            .run(move |event, event_loop_window_target| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => Emulator::exit(event_loop_window_target),
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => Emulator::on_input(&mut self.cpu, &event),
                Event::AboutToWait => {
                    Emulator::update(&mut self.cpu, &mut self.window, &mut self.screen_renderer)
                }
                _ => (),
            })
    }

    fn exit<T>(target: &EventLoopWindowTarget<T>) {
        println!("Exiting...");
        target.exit();
    }

    fn update(cpu: &mut Cpu, window: &mut Window, screen_renderer: &mut Pixels) {
        cpu.run_instruction();
        cpu.tick_timers();

        if cpu.draw_flag {
            for (i, pixel) in screen_renderer.frame_mut().chunks_exact_mut(4).enumerate() {
                let color = if cpu.screen[i] > 0 {
                    [0xFF, 0xFF, 0xFF, 0xFF]
                } else {
                    [0x00, 0x00, 0x00, 0x00]
                };
                pixel.copy_from_slice(&color);
            }
            screen_renderer.render().unwrap_or_else(|err| {
                println!("Error at screen_renderer::render(): {}", err);
                return;
            });
            window.request_redraw();
        }
    }

    fn on_input(cpu: &mut Cpu, event: &KeyEvent) {
        if let Some(chip8_key) = Emulator::get_chip8_key_code(&event.logical_key) {
            cpu.keys[chip8_key as usize] = event.state == ElementState::Pressed;
        }
    }

    fn get_chip8_key_code(key: &Key) -> Option<u8> {
        match key.as_ref() {
            Key::Character("1") => Some(0x1),
            Key::Character("2") => Some(0x2),
            Key::Character("3") => Some(0x3),
            Key::Character("4") => Some(0xC),
            Key::Character("q") => Some(0x4),
            Key::Character("w") => Some(0x5),
            Key::Character("e") => Some(0x6),
            Key::Character("r") => Some(0xD),
            Key::Character("a") => Some(0x7),
            Key::Character("s") => Some(0x8),
            Key::Character("d") => Some(0x9),
            Key::Character("f") => Some(0xE),
            Key::Character("z") => Some(0xA),
            Key::Character("x") => Some(0x0),
            Key::Character("c") => Some(0xB),
            Key::Character("v") => Some(0xF),
            _ => None,
        }
    }
}
