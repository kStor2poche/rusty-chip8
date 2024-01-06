use std::error::Error;
use crate::errors::InvalidAccessError;

pub trait Memory16Bit {
    fn get(&self, addr: u16, len: u16) -> Result<&[u8], Box<dyn Error>>;
    fn set(&mut self, addr: u16, content: &Vec<u8>) -> Result<(), Box<dyn Error>>;
}

pub struct Chip8Mem {
    ram: Vec<u8>,
}

impl Chip8Mem {
    pub fn new() -> Self {
        Self { ram: vec![0; 4096] } // 4K ram
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

    fn set(&mut self, addr: u16, content: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        if addr as usize + content.len() > 0xFFF {
            return Err(Box::new(InvalidAccessError::new(format!(
                        "Cannot set {} bytes starting from {}, too big for emulated memory !", content.len(), addr)
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
}
