use std::env;

use chipo::emulator::emulator::Emulator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Expected 1 argument, got {} instead.", args.len() - 1);
    }

    let mut emulator = Emulator::new();
    emulator.load_rom(&args[1]).unwrap_or_else(|err| {
        println!("Cannot open rom! {}", err);
    });

    emulator.run();
}
