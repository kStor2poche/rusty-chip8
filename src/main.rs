use std::error::Error;

mod systems;
mod mem;
mod display;
mod errors;
mod debug;

use systems::{System, Chip8};
use minifb;

fn open(path: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(std::fs::read(path)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("Usage : emu [CHIP-8 program]");
    // might use clap later instead

    let program_data = open(path)?;

    let mut chip8 = Chip8::init();
    let _ = chip8.load_program(program_data);
    println!("{}", chip8);
    for _ in 0..60 {
        match chip8.exec_instruction() {
            Ok(_) => {
                println!("{}", chip8);
                // TODO: wait for user input before going to next frame
                // and/or have a key to toggle fullspeed/debug
                // Anyway, we'll see how it goes when we implement minifb
            },
            Err(err) => {
                return Err(err);
            }
        };
    }

    Ok(())
}
