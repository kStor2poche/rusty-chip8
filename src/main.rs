use std::error::Error;

mod systems;
mod mem;
mod io;
mod errors;
mod debug;

fn open_bytes(path: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(std::fs::read(path)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("Usage : emu [CHIP-8 program]");
    // might use clap later instead to discern between systems and have some debug options

    let program_data = open_bytes(path)?;

    match io::chip8_io_loop(&program_data) {
        Ok(_) => (),
        Err(err) => return Err(err),
    };
    Ok(())
}
