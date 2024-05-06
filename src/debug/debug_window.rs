use crate::emulator::cpu::Cpu;

pub struct DebugWindow {
    is_open: bool,
    current_opcode: u16,
    registers: [u8; 16],
    index_register: u16,
    counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    keys: [bool; 16],
    draw_flag: bool,
}

impl DebugWindow {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_opcode: 0,
            registers: [0; 16],
            index_register: 0,
            counter: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: vec![],
            keys: [false; 16],
            draw_flag: false,
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn update(&mut self, cpu: &Cpu) {
        self.current_opcode = cpu.opcode;
        self.counter = cpu.pc;
        self.registers = cpu.v;
    }

    pub fn redraw(&mut self, ctx: &egui::Context) {
        egui::Window::new("Debug window")
            .open(&mut self.is_open)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.strong("OPCODE");
                    ui.monospace(format!("{:#06X}", self.current_opcode));
                    ui.add_space(16.0);
                    ui.strong("PC");
                    ui.monospace(format!("{:#06X}", self.counter));
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.heading("Registers");
                ui.add_space(4.0);

                ui.columns(4, |columns| {
                    for (i, register) in self.registers.iter().enumerate() {
                        columns[i / 4].group(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("{:#X}", i).replace("0x", ""));
                                ui.monospace(format!("{:#06X}", register));
                            });
                        });
                    }
                });
            });
    }
}
