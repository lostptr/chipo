use super::cpu::{Cpu, PROGRAM_START, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::debug::egui_winit_framework::EguiWinitFramework;
use pixels::{Pixels, SurfaceTexture};
use std::{
    fs::File,
    io::{self, Read},
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const SCALING: usize = 8;

pub struct Emulator {
    event_loop: EventLoop<()>,
    window: Window,
    screen_renderer: Pixels,
    cpu: Cpu,
    framework: EguiWinitFramework,
}

impl Emulator {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(
                (SCREEN_WIDTH * SCALING) as f64,
                (SCREEN_HEIGHT * SCALING) as f64,
            );
            WindowBuilder::new()
                .with_title("Chipo Emulator")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let (screen_renderer, framework) = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels =
                Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap();
            let framework = EguiWinitFramework::new(
                &event_loop,
                window_size.width,
                window_size.height,
                window.scale_factor() as f32,
                &pixels,
            );

            (pixels, framework)
        };

        Self {
            window,
            event_loop,
            screen_renderer,
            cpu: Cpu::new(),
            framework,
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

    pub fn run(mut self) {
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => Emulator::exit(control_flow),
                Event::WindowEvent { event, .. } => {
                    Emulator::on_input(&mut self.cpu, &event, &mut self.framework)
                }
                Event::MainEventsCleared => Emulator::update(
                    &mut self.cpu,
                    &mut self.window,
                    &mut self.screen_renderer,
                    &mut self.framework,
                ),
                _ => (),
            }
        })
    }

    fn exit(target: &mut ControlFlow) {
        println!("Exiting...");
        target.set_exit();
    }

    fn update(
        cpu: &mut Cpu,
        window: &mut Window,
        screen_renderer: &mut Pixels,
        framework: &mut EguiWinitFramework,
    ) {
        cpu.run_instruction();
        cpu.tick_timers();

        framework.prepare(window);

        if cpu.draw_flag {
            framework.update_cpu(cpu);
            Emulator::draw_frame(cpu, screen_renderer, framework);
            window.request_redraw();
        }
    }

    fn draw_frame(cpu: &mut Cpu, screen_renderer: &mut Pixels, framework: &mut EguiWinitFramework) {
        for (i, pixel) in screen_renderer.frame_mut().chunks_exact_mut(4).enumerate() {
            let color = if cpu.screen[i] > 0 {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&color);
        }
        let render_result = screen_renderer.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
            framework.render(encoder, render_target, context);
            Ok(())
        });

        if let Err(err) = render_result {
            println!("oh no!! {}", err);
        }
    }

    fn on_input(cpu: &mut Cpu, event: &WindowEvent, framework: &mut EguiWinitFramework) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    if let Some(chip8_key) = Emulator::get_chip8_key_code(&keycode) {
                        cpu.keys[chip8_key as usize] = input.state == ElementState::Pressed;
                    } else {
                        // make gui accept input event
                        framework.handle_event(event);
                    }
                }
            },
            _ => framework.handle_event(event),
        }
    }

    fn get_chip8_key_code(key: &VirtualKeyCode) -> Option<u8> {
        match key {
            VirtualKeyCode::Key1 => Some(0x1),
            VirtualKeyCode::Key2 => Some(0x2),
            VirtualKeyCode::Key3 => Some(0x2),
            VirtualKeyCode::Key4 => Some(0xC),
            VirtualKeyCode::Q => Some(0x4),
            VirtualKeyCode::W => Some(0x5),
            VirtualKeyCode::E => Some(0x6),
            VirtualKeyCode::R => Some(0xD),
            VirtualKeyCode::A => Some(0x7),
            VirtualKeyCode::S => Some(0x8),
            VirtualKeyCode::D => Some(0x9),
            VirtualKeyCode::F => Some(0xE),
            VirtualKeyCode::Z => Some(0xA),
            VirtualKeyCode::X => Some(0x0),
            VirtualKeyCode::C => Some(0xB),
            VirtualKeyCode::V => Some(0xF),
            _ => None,
        }
    }
}
