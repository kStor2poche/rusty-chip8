use std::error::Error;
use crate::{errors::InvalidAccessError, systems::{CHIP8_DISP_BUF_ADDR, CHIP8_DISP_BUF_LEN, CHIP8_DISP_WIDTH, CHIP8_DISP_HEIGHT}};

pub trait Memory16Bit {
    fn get(&self, addr: u16, len: u16) -> Result<&[u8], Box<dyn Error>>;
    fn set(&mut self, addr: u16, content: &[u8]) -> Result<(), Box<dyn Error>>;
    fn set_byte(&mut self, addr: u16, content: u8) -> Result<(), Box<dyn Error>>;
    fn dump(&self) -> &Vec<u8>;
}

pub struct Chip8Mem {
    ram: Vec<u8>,
}

impl Chip8Mem {
    pub fn new() -> Self {
        Self { ram: vec![0; 4096] } // 4K ram
    }
    pub fn load_sprite(&mut self, sprite: &[u8], x_uncapped: u8, y_uncapped: u8, n: u8) -> Result<bool, Box<dyn Error>>{
        // not too bad in the end :)
        let x = (x_uncapped & CHIP8_DISP_WIDTH as u8) - 1;
        let y = (y_uncapped & CHIP8_DISP_HEIGHT as u8) - 1;
        let mut fb = self.get(CHIP8_DISP_BUF_ADDR, CHIP8_DISP_BUF_LEN)
                         .unwrap()
                         .to_owned();
        let mut flag = false;
        fb.iter_mut()
          .enumerate()
          .filter(|(i, _b)| {
              let fb_x = i % (CHIP8_DISP_WIDTH as usize / 8);
              let fb_y = i / (CHIP8_DISP_WIDTH as usize / 8);
              x as usize/8 <= fb_x
                  && fb_x <= (x as usize/8)+1
                  && y as usize <= fb_y 
                  && fb_y < (y + n) as usize
          })
          .enumerate()
          .for_each(|(j,(_i, b))| {
              if (x as usize)/8 == 7 {
                  *b ^= sprite[j].checked_shr((x%8).into()).unwrap_or(0);
                  flag |= *b ^ (sprite[j].checked_shr((x%8).into()).unwrap_or(0)) > 0;
                  return;
              }
              if j%2 == 0 {
                  *b ^= sprite[j/2].checked_shr((x%8).into()).unwrap_or(0);
                  flag |= *b ^ (sprite[j/2].checked_shr((x%8).into()).unwrap_or(0)) > 0;
              } else {
                  *b ^= sprite[j/2].checked_shl((8-(x%8)).into()).unwrap_or(0);
                  flag |= *b ^ (sprite[j/2].checked_shl((8-(x%8)).into()).unwrap_or(0)) > 0;
              }
          });
        self.set(CHIP8_DISP_BUF_ADDR, &fb)?;
        Ok(flag)
    }
}

impl Memory16Bit for Chip8Mem {
    fn get(&self, addr: u16,len: u16) -> Result<&[u8], Box<dyn Error>> {
        let res = &self.ram.get(addr as usize..(addr as usize + len as usize));
        match res {
            Some(_) => Ok(res.unwrap()),
            None => Err(Box::new(InvalidAccessError::new(format!("Address 0x{:X} unreachable", addr))))
        }
    }

    fn set(&mut self, addr: u16, content: &[u8]) -> Result<(), Box<dyn Error>> {
        if addr as usize + content.len() - 1 > 0xFFF {
            return Err(Box::new(InvalidAccessError::new(format!(
                        "Cannot set 0x{:X} bytes starting from 0x{:03X}, too big for emulated memory !", content.len(), addr)
                    )))
        }

        self.ram = self.ram.iter()
                           .enumerate()
                           .map(|(i, byte)| 
                                if i>=addr as usize && i< addr as usize+content.len() {
                                    content[i - addr as usize]
                                } else {
                                    *byte
                                })
                           .collect(); // might just do it imperative, will save the copy
                                       // and allow us to work only on the rewritten part
                                       // and it'll be ok since not paralellized
        Ok(())
    }

    fn set_byte(&mut self, addr: u16, content: u8) -> Result<(), Box<dyn Error>> {
        if addr as usize > 0xFFF {
            return Err(Box::new(InvalidAccessError::new(format!(
                        "Cannot set a bytes starting from 0x{:03X}, too big for emulated memory !", addr)
                    )))
        }

        self.ram[addr as usize] = content;
        Ok(())
    }

    fn dump(&self) -> &Vec<u8> {
        &self.ram
    }
}
