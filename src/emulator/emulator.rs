use super::cpu::{Cpu, PROGRAM_START, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use std::{
    fs::File,
    io::{self, Read},
    rc::Rc,
    sync::Arc,
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

const SCALING: usize = 8;

pub struct Emulator<'win> {
    event_loop: EventLoop<()>,
    screen_renderer: Pixels<'win>,
    input: WinitInputHelper,
    window: Arc<Window>,
    cpu: Cpu,
    frames: u16,
}

impl<'win> Emulator<'win> {
    pub fn new() -> Emulator<'win> {
        let event_loop = EventLoop::new().unwrap(); // todo: handle this unwrap
        let window = {
            let size = LogicalSize::new(
                (SCREEN_WIDTH * SCALING) as f64,
                (SCREEN_HEIGHT * SCALING) as f64,
            );

            Arc::new(
                WindowBuilder::new()
                    .with_title("Chipo Emulator")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .unwrap(), // todo: handle this unwrap
            )
        };

        let screen_renderer = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());

            Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
        };

        Self {
            event_loop,
            screen_renderer,
            window,
            input: WinitInputHelper::new(),
            cpu: Cpu::new(),
            frames: 0,
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

    pub fn run(emulator: &mut Emulator) {
        let result = emulator.event_loop.run(|event, elwt| {
            if let Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } = event
            {
                // draw...
            }

            if emulator.input.update(&event) {
                if emulator.input.close_requested() {
                    elwt.exit();
                    return;
                }

                // emulator input handler...
            }

            // update
            emulator.update();
        });
        // let a = self.event_loop.run(move |event, elwt| {
        //     match event {
        //         Event::WindowEvent {
        //             event: WindowEvent::CloseRequested,
        //             ..
        //         } => Emulator::exit(control_flow),
        //         Event::WindowEvent { event, .. } => Emulator::on_input(&mut self.cpu, &event),
        //         // todo: why not use mutable self in emulator.update ?
        //         Event::MainEventsCleared => {
        //             Emulator::update(&mut self.cpu, &mut self.window, &mut self.screen_renderer, &mut self.frames);
        //         },
        //         _ => (),
        //     }
        // })
    }

    fn exit(target: &mut ControlFlow) {
        println!("Exiting...");
        target.set_exit();
    }

    fn update(mut self) {
        self.cpu.run_instruction();
        self.frames += 1;
        if self.frames > 30 {
            self.cpu.tick_timers();
            self.frames = 0;
        }

        if self.cpu.draw_flag {
            self.draw_frame();
            self.window.request_redraw();
        }
    }

    fn draw_frame(mut self) {
        for (i, pixel) in self.screen_renderer.frame_mut().chunks_exact_mut(4).enumerate() {
            let color = if self.cpu.screen[i] > 0 {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&color);
        }
        let render_result = self.screen_renderer.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
            Ok(())
        });

        if let Err(err) = render_result {
            println!("oh no!! {}", err);
        }
    }

    fn on_input(cpu: &mut Cpu, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    if let Some(chip8_key) = Emulator::get_chip8_key_code(&keycode) {
                        cpu.keys[chip8_key as usize] = input.state == ElementState::Pressed;
                    }
                }
            }
            _ => {}
        }
    }

    fn get_chip8_key_code(key: &VirtualKeyCode) -> Option<u8> {
        match key {
            VirtualKeyCode::Key1 => Some(0x1),
            VirtualKeyCode::Key2 => Some(0x2),
            VirtualKeyCode::Key3 => Some(0x3),
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
