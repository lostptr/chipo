use std::env;

use chipo::emulator::{emu2::Emu2, options::EmulatorOptions};


fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Expected 1 argument, got {} instead.", args.len() - 1);
    }

    let mut emu2 = Emu2::new(EmulatorOptions {
        scaling: 8,
    });
    emu2.load_rom(&args[1]).unwrap_or_else(|err| {
        println!("Cannot open rom! {}", err);
    });
    emu2.run();
}
