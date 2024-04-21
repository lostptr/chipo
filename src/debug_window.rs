// use std::fmt::format;

// pub struct DebugWindow {
//     current_opcode: u16,
//     registers: [u8; 16],
//     index_register: u16,
//     counter: u16,
//     delay_timer: u8,
//     sound_timer: u8,
//     stack: Vec<u16>,
//     keys: [bool; 16],
//     draw_flag: bool,
// }

// impl Default for DebugWindow {
//     fn default() -> Self {
//         Self {
//             current_opcode: 0,
//             registers: [0; 16],
//             index_register: 0,
//             counter: 0,
//             delay_timer: 0,
//             sound_timer: 0,
//             stack: vec![],
//             keys: [false; 16],
//             draw_flag: false,
//         }
//     }
// }

// impl eframe::App for DebugWindow {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
//                 ui.strong("OPCODE");
//                 ui.monospace(format!("{:#06X}", self.current_opcode));
//                 ui.add_space(16.0);
//                 ui.strong("PC");
//                 ui.monospace(format!("{:#06X}", self.counter));
//             });

//             ui.add_space(4.0);
//             ui.separator();
//             ui.add_space(4.0);

//             ui.heading("Registers");
//             ui.add_space(4.0);

//             ui.columns(4, |columns| {
//                 for (i, register) in self.registers.iter().enumerate() {
//                     columns[i / 4].group(|ui| {
//                         ui.horizontal(|ui| {
//                             ui.strong(format!("{:#X}", i).replace("0x", ""));
//                             ui.monospace(format!("{:#06X}", register));
//                         });
//                     });
//                 }
//             });
//         });
//     }
// }
