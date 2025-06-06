use std::{
    fs::File,
    io::{self, Read},
};

use log::{debug, error};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    window::WindowBuilder,
};

use super::{
    cpu::{Cpu, SCREEN_HEIGHT, SCREEN_WIDTH},
    options::EmulatorOptions,
};

pub struct Emu2 {
    options: EmulatorOptions,
    rom: Option<Vec<u8>>,
}

impl Emu2 {
    pub fn new(options: EmulatorOptions) -> Self {
        Self { options, rom: None }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), io::Error> {
        let program_data = Emu2::load_rom_file(path)?;
        self.rom = Some(program_data);
        Ok(())
    }

    fn load_rom_file(path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        let bytes_read = file.read_to_end(&mut buffer)?;

        println!("Loaded '{}' ({} bytes read)", path, bytes_read);

        Ok(buffer)
    }

    pub fn run(self) {
        let mut cpu = Cpu::new();
        if let Some(rom) = self.rom {
            cpu.load_rom(&rom);
        } else {
            panic!("No rom was loaded!");
        }

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let window = {
            let size = LogicalSize::new(
                (SCREEN_WIDTH as f64) * (self.options.scaling as f64),
                (SCREEN_HEIGHT as f64) * (self.options.scaling as f64),
            );
            WindowBuilder::new()
                .with_title("Hello Pixels")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut screen_renderer = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
            // todo: handle error
        };

        let mut frame_count_timer = 0;
        let res = event_loop.run(|event, event_handler| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event_handler.exit(),
                    WindowEvent::KeyboardInput { mut event, .. } => {
                        Emu2::input(&mut event, &mut cpu)
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(error) = Emu2::draw(&mut screen_renderer, &mut cpu) {
                            println!("error: {}", error);
                            event_handler.exit();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            cpu.run_instruction();
            frame_count_timer += 1;
            if frame_count_timer > 30 {
                cpu.tick_timers();
            }

            if cpu.draw_flag {
                window.request_redraw();
            }
        });

        if let Err(error) = res {
            error!("{}", error);
        }
    }

    fn draw(screen_renderer: &mut Pixels, cpu: &mut Cpu) -> std::result::Result<(), pixels::Error> {
        for (i, pixel) in screen_renderer.frame_mut().chunks_exact_mut(4).enumerate() {
            let color = if cpu.screen[i] > 0 {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&color);
        }

        screen_renderer.render()
    }

    fn input(input: &mut KeyEvent, cpu: &mut Cpu) {
        if let Key::Character(keystr) = &input.logical_key {
            if let Some(chip8_key) = Emu2::get_chip8_key_code(&keystr) {
                debug!(
                    "keyboard event: {} -> {}",
                    &keystr,
                    if input.state.is_pressed() {
                        "pressed"
                    } else {
                        "released"
                    }
                );
                cpu.keys[chip8_key as usize] = input.state.is_pressed();
            }
        }
    }

    fn get_chip8_key_code(key: &str) -> Option<u8> {
        match key {
            "1" => Some(0x1),
            "2" => Some(0x2),
            "3" => Some(0x3),
            "4" => Some(0xC),
            "q" => Some(0x4),
            "w" => Some(0x5),
            "e" => Some(0x6),
            "r" => Some(0xD),
            "a" => Some(0x7),
            "s" => Some(0x8),
            "d" => Some(0x9),
            "f" => Some(0xE),
            "z" => Some(0xA),
            "x" => Some(0x0),
            "c" => Some(0xB),
            "v" => Some(0xF),
            _ => None,
        }
    }
}
