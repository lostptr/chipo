use chipo::emulator::emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    let rom_path: &str = "roms/pong";
    emulator.load_rom(rom_path).unwrap_or_else(|err| {
        println!("Cannot open rom! {}", err);
    });

    emulator.run().unwrap_or_else(|err| {
        println!("Oh no! {}", err);
    });
}
