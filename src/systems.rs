use std::error::Error;
use rand::{Rng, rngs::ThreadRng};

use crate::mem::{Chip8Mem, Memory16Bit};
use crate::errors::{InvalidInstructionError, ProgramLoadingError};

pub trait System {
    fn init() -> Self;
    fn load_program(&mut self, program_data: &Vec<u8>) -> Result<(), Box<dyn Error>>;
    fn exec_instruction(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct Chip8 {
    i: u16,
    sp: u16,
    pc: u16,
    v: Vec<u8>,
    delay: u8,
    sound: u8,
    ram: Chip8Mem,
    rng: ThreadRng,
}

const CHIP8_PC_START: u16 = 0x200;
const CHIP8_MAX_PROG_SIZE: u16 = CHIP8_STACK_BASE_ADDR - CHIP8_PC_START;
pub const CHIP8_DISP_BUF_ADDR: u16 = 0xF00;
pub const CHIP8_DISP_BUF_LEN: u16 = 0x100;
pub const CHIP8_DISP_WIDTH: u16 = 64;
pub const CHIP8_DISP_HEIGHT: u16 = 32;
const CHIP8_STACK_BASE_ADDR: u16 = 0xEA0;

impl Chip8 {
    pub fn get_state(&self) -> (u16, u16, u16, &Vec<u8>, u8, u8, &Chip8Mem) {
        (self.i, self.sp, self.pc, &self.v, self.delay, self.sound, &(self.ram))
    }
    pub fn get_mem(&mut self) -> &mut Chip8Mem {
        &mut self.ram
    }
}

impl System for Chip8 {
    fn init() -> Self {
        Self {
            i: 0,
            sp: CHIP8_STACK_BASE_ADDR,
            pc: CHIP8_PC_START,
            v: vec![0; 16],
            delay: 0,
            sound: 0,
            ram: Chip8Mem::new(),
            rng: rand::thread_rng(),
        }
    }

    fn load_program(&mut self, program_data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        if program_data.len() > CHIP8_MAX_PROG_SIZE as usize {
            return Err(Box::new(ProgramLoadingError::new(format!(
                        "Program too long, {} > 3,328 KB", program_data.len())
                        )))
        }
        Ok(self.ram.set(CHIP8_PC_START, program_data).unwrap()) // shouldn't return an Err, so we
                                                                // unwrap and panic if something
                                                                // real bad happens
    }

    fn exec_instruction(&mut self) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        let opcode = self.ram.get(self.pc, 2)
                             .map(|op| (op[0] >> 4, op[0] & 0x0F, op[1] >> 4, op[1] & 0x0F))
                             .expect(&format!("Couldn't read opcode at 0x{:X}", self.pc).to_string());
        match opcode {
            // 0 - return subroutine (RTS) and display clear (CLS)
            (0x0, b, m, l) => {
                match (b, m, l) {

                    (0x0, 0xE, 0x0) => { // CLS
                        self.ram.set(CHIP8_DISP_BUF_ADDR, &vec![0; CHIP8_DISP_BUF_LEN as usize]).unwrap();
                    },

                    (0x0, 0xE, 0xE) => { // RTS
                        self.sp -= 2;
                        if self.sp < CHIP8_STACK_BASE_ADDR {
                            return Err(Box::new(InvalidInstructionError::new(
                                       format!("tried to return from main subroutine (SP decreased below {:X})",
                                               CHIP8_STACK_BASE_ADDR)
                                       )));
                        }
                        let addr_bytes = self.ram.get(self.pc, 2).unwrap();
                        self.pc = u16::from_be_bytes([addr_bytes[0], addr_bytes[1]]);
                    },

                    err => return Err(Box::new(InvalidInstructionError::new(
                                      format!("wrong operand \"0x{:X}\" for opcode 0x0 (should be 0x0E0 or 0x0EE)",
                                              u16::from_be_bytes([err.0, err.1 << 4 + err.2]))
                                      ))),
                };
            },

            // 1 - JMP
            (0x1, b, m, l) => {
                self.pc = u16::from_be_bytes([b , (m << 4) + l]);
                return Ok(())
            },

            // 2 - CALL
            (0x2, b, m, l) => {
                if self.sp >= CHIP8_DISP_BUF_ADDR {
                    return Err(Box::new(InvalidInstructionError::new("exceeded stack frame limit")));
                }
                let _ = self.ram.set(self.sp, &self.pc.to_be_bytes().to_vec());
                self.sp += 2;
                self.pc = u16::from_be_bytes([b , (m << 4) + l]);
            },

            // 3 - SKIP.EQ direct
            (0x3, x, b, l) => {
                if self.v[x as usize] == (b << 4)+l {
                    self.pc = (self.pc + 2) & 0b0000111111111111;
                }
            },

            // 4 - SKIP.NE direct
            (0x4, x, b, l) => {
                if self.v[x as usize] != (b << 4)+l {
                    self.pc = (self.pc + 2) & 0b0000111111111111;
                }
            },

            // 5 - SKIP.EQ register
            (0x5, x, y, 0x0) => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc = (self.pc + 2) & 0b0000111111111111;
                }
            },

            // 6 - SET direct
            (0x6, x, b, l) => self.v[x as usize] = (b<<4) + l,

            // 7 - INCR direct
            (0x7, x, b, l) => self.v[x as usize] += (b<<4) + l, // carry flag unchanged

            // 8 - Register based ops
            (0x8, x, y, op) => {
                match op {
                    0x0 => self.v[x as usize] = self.v[y as usize], // MOV
                    0x1 => self.v[x as usize] |= self.v[y as usize], // OR
                    0x2 => self.v[x as usize] &= self.v[y as usize], // AND
                    0x3 => self.v[x as usize] ^= self.v[y as usize], // XOR
                    0x4 => { // ADD
                        let res = self.v[x as usize].overflowing_add(self.v[y as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = res.1 as u8;
                    },
                    0x5 => { // SUB
                        let res = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = res.1 as u8;
                    },
                    0x6 => { // SHR
                        self.v[0xF] = self.v[x as usize] & 0x01;
                        self.v[x as usize] >>= 1;
                    },
                    0x7 => { // RSUB
                        let res = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = res.1 as u8;
                    },
                    0xE => { // SHL
                        self.v[0xF] = self.v[x as usize] & 0xF0;
                        self.v[x as usize] <<= 1
                    },

                    err => return Err(Box::new(InvalidInstructionError::new(
                                      format!("wrong operation \"0x{:X}\" for opcode 0x8 (should be 0x0 -> 0x7 or 0xE)",
                                              err)
                                      ))),
                }
            },

            // 9 - SKIP.NE register
            (0x9, x, y, 0x0) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc = (self.pc + 2) & 0b0000111111111111;
                }
            },

            // A - SETI
            (0xA, b, m, l) => self.i = u16::from_be_bytes([b , (m << 4) + l]),

            // B - JMP relative
            (0xB, b, m, l) => {
                self.pc = (self.v[0] as u16 + u16::from_be_bytes([b , (m << 4) + l])) & 0b0000111111111111;
                return Ok(())
            },

            // C - RAND (VX = rand() & BL)
            (0xC, x, b, l) => self.v[x as usize] = self.rng.gen_range(0..=255) & ((b << 4) + l),

            // D - DISP (draws sprite @ coord VX,VY, N pixels high, see wikipedia.org for exact spec)
            (0xD, x, y, n) => {
                let sprite;
                match self.ram.get(self.i, n as u16) {
                    Ok(slice) => sprite = slice.iter().cloned().collect::<Vec<u8>>(),
                    Err(_err) => return Ok(()),
                }
                for j in 0..n as u16 {
                    let _ = self.ram.set(CHIP8_DISP_BUF_ADDR + CHIP8_DISP_WIDTH * (j + self.v[y as usize] as u16) + self.v[x as usize] as u16,
                                         &sprite.get(j as usize..j as usize).unwrap().to_vec());
                    // TODO: set VF if a pixel is flipped. Maybe write another set function
                    // for the video buffer ? (self.v[0xf] |= self.ram.set_disp_buf([...]))
                    // will have to rewrite because it is not an write but a XOR
                    // so that would rather mean a ram.xor function
                }
            },

            // E - INPT checking
            (0xE, b, m, l) => todo!("{}{}{}", b, m, l),

            // F - INPT & SND related things
            (0xF, x, op_b, op_l) => {
                match (x, (op_b << 4) + op_l) {
                    (x, _) => todo!("{}", x),
                }
            }

            err => return Err(Box::new(InvalidInstructionError::new(
                        format!("invalid opcode \"0x{:X}\"",
                                u16::from_be_bytes([err.0 << 4 + err.1, err.2 << 4 + err.3]))
                        ))),
        };
        self.pc = (self.pc + 2) & 0b0000111111111111;
        Ok(())
    }
}
