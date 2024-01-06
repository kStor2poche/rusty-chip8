use std::error::Error;

use minifb::{Key, Window, WindowOptions};

use crate::{systems::{Chip8, System, self, CHIP8_DISP_BUF_ADDR}, mem::Memory16Bit};

fn fb_from_bytes(bytes: &[u8]) -> Vec<u32> {
    bytes.iter()
         .map(|c| vec![ if c & 0b10000000 == 0b10000000 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b01000000 == 0b01000000 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00100000 == 0b00100000 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00010000 == 0b00010000 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00001000 == 0b00001000 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00000100 == 0b00000100 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00000010 == 0b00000010 {0xFFFFFFFF} else {0xFF000000},
                        if c & 0b00000001 == 0b00000001 {0xFFFFFFFF} else {0xFF000000},])
         .flatten().collect::<Vec<u32>>()
}

pub fn chip8_display_loop(program_data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
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
    // limit framerate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let _ = chip8.get_mem().set(CHIP8_DISP_BUF_ADDR, &vec![0xA0u8]);

    while window.is_open() && !window.is_key_down(Key::Q) {
        window.update_with_buffer(&fb_from_bytes(chip8.get_mem()
                                                      .get(systems::CHIP8_DISP_BUF_ADDR,
                                                           systems::CHIP8_DISP_BUF_LEN)
                                                      .unwrap()),
                                  systems::CHIP8_DISP_WIDTH as usize,
                                  systems::CHIP8_DISP_HEIGHT as usize)
              .unwrap();
        if !window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            continue;
        }
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
