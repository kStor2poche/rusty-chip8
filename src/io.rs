use std::error::Error;

use minifb::{Key, Window, WindowOptions};

use crate::{systems::{Chip8, System, self}, mem::Memory16Bit};

const MINIFB_COLOR_BG: u32 = 0xFF221110;
const MINIFB_COLOR_FG: u32 = 0xFFFF9900;

fn minifb_from_bytes(bytes: &[u8]) -> Vec<u32> {
    bytes.iter()
         .flat_map(|c| vec![ if c & 0b10000000 == 0b10000000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b01000000 == 0b01000000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00100000 == 0b00100000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00010000 == 0b00010000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00001000 == 0b00001000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00000100 == 0b00000100 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00000010 == 0b00000010 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                             if c & 0b00000001 == 0b00000001 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},])
         .collect::<Vec<u32>>()
}

pub fn chip8_get_key(window: &Window, key_byte: u8) -> bool {
    let key = match key_byte {
        0x0 => Key::NumPad0,
        0x1 => Key::NumPad1,
        0x2 => Key::NumPad2,
        0x3 => Key::NumPad3,
        0x4 => Key::NumPad4,
        0x5 => Key::NumPad5,
        0x6 => Key::NumPad6,
        0x7 => Key::NumPad7,
        0x8 => Key::NumPad8,
        0x9 => Key::NumPad9,
        0xA => Key::NumPadDot,
        0xB => Key::NumPadEnter,
        0xC => Key::NumPadPlus,
        0xD => Key::NumPadMinus,
        0xE => Key::NumPadAsterisk,
        0xF => Key::NumPadSlash,
        _ => return false,
    };
    window.is_key_down(key)
}

pub fn chip8_get_any_key(window: &Window) -> Option<u8> {
    let keys = window.get_keys_pressed(minifb::KeyRepeat::Yes);
    if let Some(key) = keys.into_iter().next() {
        match key {
            Key::NumPad0 => return Some(0x0),
            Key::NumPad1 => return Some(0x1),
            Key::NumPad2 => return Some(0x2),
            Key::NumPad3 => return Some(0x3),
            Key::NumPad4 => return Some(0x4),
            Key::NumPad5 => return Some(0x5),
            Key::NumPad6 => return Some(0x6),
            Key::NumPad7 => return Some(0x7),
            Key::NumPad8 => return Some(0x8),
            Key::NumPad9 => return Some(0x9),
            Key::NumPadDot => return Some(0xA),
            Key::NumPadEnter => return Some(0xB),
            Key::NumPadPlus => return Some(0xC),
            Key::NumPadMinus => return Some(0xD),
            Key::NumPadAsterisk => return Some(0xE),
            Key::NumPadSlash => return Some(0xF),
            _ => return None,
        };
    }
    None
}

pub fn chip8_io_loop(program_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::init();
    let _ = chip8.load_program(program_data);

    let mut window = Window::new(
            "rusty chip8 !",
            systems::CHIP8_DISP_WIDTH as usize,
            systems::CHIP8_DISP_HEIGHT as usize,
            WindowOptions { borderless: false,
                            title: true,
                            resize: true,
                            scale: minifb::Scale::X8,
                            scale_mode: minifb::ScaleMode::AspectRatioStretch,
                            topmost: false,
                            transparency: false,
                            none: false, }
        ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(1429)));

    while window.is_open() && !window.is_key_down(Key::Q) {
        window.update_with_buffer(&minifb_from_bytes(chip8.get_mem()
                                                      .get(systems::CHIP8_DISP_BUF_ADDR,
                                                           systems::CHIP8_DISP_BUF_LEN)
                                                      .unwrap()),
                                  systems::CHIP8_DISP_WIDTH as usize,
                                  systems::CHIP8_DISP_HEIGHT as usize)
              .unwrap();
        /*if !window.is_key_pressed(Key::N, minifb::KeyRepeat::Yes) {
            continue;
        }*/
        match chip8.exec_instruction(Some(&window)) {
            Ok(_) => {
                /*println!("{}", chip8);
                if window.is_key_down(Key::D) {
                    println!("{}", chip8.get_mem());
                }*/
            },
            Err(err) => {
                println!("\x1b[31;1;4mCore dumped :\x1b[0m \n");
                println!("{}", chip8);
                println!("{}", chip8.get_mem());
                return Err(err);
            }
        };
    }
    Ok(())
}
