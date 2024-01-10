use std::error::Error;
use std::time::{Instant, Duration};
use minifb::Window;
use rand::{Rng, rngs::ThreadRng};

use crate::io::{chip8_get_key, chip8_get_any_key};
use crate::mem::{Chip8Mem, Memory16Bit};
use crate::errors::{InvalidInstructionError, ProgramLoadingError, UnvavailableIOError, InvalidAccessError};

pub trait System {
    fn init() -> Self;
    fn load_program(&mut self, program_data: &Vec<u8>) -> Result<(), Box<dyn Error>>;
    fn exec_instruction(&mut self, window: Option<&Window>) -> Result<(), Box<dyn Error>>;
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
    last_frame: Instant,
}

const CHIP8_PC_START: u16 = 0x200;
const CHIP8_MAX_PROG_SIZE: u16 = CHIP8_STACK_BASE_ADDR - CHIP8_PC_START;
pub const CHIP8_DISP_BUF_ADDR: u16 = 0xF00;
pub const CHIP8_DISP_BUF_LEN: u16 = 0x100;
pub const CHIP8_DISP_WIDTH: u16 = 64;
pub const CHIP8_DISP_HEIGHT: u16 = 32;
const CHIP8_STACK_BASE_ADDR: u16 = 0xEA0;
const CHIP8_FONT_START: u16 = 0x50;
const CHIP8_FONT_HEIGHT: u8 = 0x5;
const CHIP8_FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F 
];
const CHIP8_INSTR_P_S: u16 = 700;

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
            last_frame: Instant::now(),
        }
    }

    fn load_program(&mut self, program_data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        if program_data.len() > CHIP8_MAX_PROG_SIZE as usize {
            return Err(Box::new(ProgramLoadingError::new(format!(
                        "Program too long, {} > 3,328 KB", program_data.len())
                        )))
        }
        self.ram.set(CHIP8_FONT_START, &CHIP8_FONT.to_vec()).unwrap();
        self.ram.set(CHIP8_PC_START, program_data).unwrap(); // shouldn't return an Err, so we
                                                             // unwrap and panic if something
                                                             // real bad happens
        Ok(())
    }

    fn exec_instruction(&mut self, window: Option<&Window>) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        if self.last_frame.elapsed() >= Duration::from_nanos(16666666) {
            self.sound = self.sound.saturating_sub(1);
            self.delay = self.delay.saturating_sub(1);
            self.last_frame = Instant::now();
        }
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
                        let addr_bytes = self.ram.get(self.sp, 2).unwrap();
                        self.pc = u16::from_be_bytes([addr_bytes[0], addr_bytes[1]]);
                    },

                    err => return Err(Box::new(InvalidInstructionError::new(
                                      format!("wrong operand 0x{:03X} for opcode 0x0 (should be 0x0E0 or 0x0EE)",
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
                return Ok(())
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
            (0x7, x, b, l) => self.v[x as usize] = self.v[x as usize].overflowing_add((b<<4) + l).0, // carry flag unchanged

            // 8 - Register based ops
            (0x8, x, y, op) => {
                match op {
                    0x0 => self.v[x as usize] = self.v[y as usize], // MOV
                    0x1 => { // OR
                        self.v[x as usize] |= self.v[y as usize];
                        self.v[0xF] = 0x00;
                    },
                    0x2 => { // AND
                        self.v[x as usize] &= self.v[y as usize];
                        self.v[0xF] = 0x00;
                    },
                    0x3 => { // XOR
                        self.v[x as usize] ^= self.v[y as usize];
                        self.v[0xF] = 0x00;
                    },
                    0x4 => { // ADD
                        let res = self.v[x as usize].overflowing_add(self.v[y as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = res.1 as u8;
                    },
                    0x5 => { // SUB
                        let res = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = 1 - (res.1 as u8);
                    },
                    0x6 => { // SHR
                        // TODO: let this be configurable
                        self.v[x as usize] = self.v[y as usize];
                        let carry = self.v[x as usize] & 0x01;
                        self.v[x as usize] >>= 1;
                        self.v[0xF] = carry;
                    },
                    0x7 => { // RSUB
                        let res = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                        self.v[x as usize] = res.0;
                        self.v[0xF] = 1 - (res.1 as u8);
                    },
                    0xE => { // SHL
                        // TODO: let this be configurable
                        self.v[x as usize] = self.v[y as usize];
                        let carry = (self.v[x as usize] & 0x80) >> 7;
                        self.v[x as usize] <<= 1;
                        self.v[0xF] = carry;
                    },

                    err => return Err(Box::new(InvalidInstructionError::new(
                                      format!("wrong operation 0x{:X} for opcode 0x8 (should be 0x0 -> 0x7 or 0xE)",
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

            // D - DISP (draws sprite @ coord VX,VY, N pixels high)
            (0xD, x, y, n) => {
                if n > 0xf {
                    return Err(Box::new(InvalidInstructionError::new(format!("Trying to draw sprite with heigh {}. Height should be between 1 and 15 both included.", n))));
                }
                let sprite;
                match self.ram.get(self.i, n as u16) {
                    Ok(slice) => sprite = slice.to_vec(),
                    Err(_err) => todo!(),
                }
                self.v[0xF] = match Chip8Mem::load_sprite(&mut self.ram,
                                                          &sprite,
                                                          self.v[x as usize],
                                                          self.v[y as usize],
                                                          n) {
                    Ok(flag) => flag as u8,
                    Err(err) => return Err(err),
                }
            },

            // E - INPT checking
            (0xE, b, m, l) => {
                match window {
                    Some(window) => {
                        match (b, m, l) {
                            (x, 0x9, 0xE) => if chip8_get_key(&window, self.v[x as usize]) {
                                self.pc += 2;
                            },
                            (x, 0xA, 0x1) => if !chip8_get_key(&window, self.v[x as usize]) {
                                self.pc += 2;
                            },
                            err => return Err(Box::new(InvalidInstructionError::new(
                                              format!("wrong operand 0x{:03X} for opcode 0xE (should be 0xE[X]9E or 0xE[X]A1)",
                                                      u16::from_be_bytes([err.0, err.1 << 4 + err.2]))
                                              ))),
                        }
                    }
                    None => return Err(Box::new(UnvavailableIOError::new(
                                "Can't fetch inputs while running headless"
                                ))),
                }
            },

            // F - MISC things
            (0xF, x, op_b, op_l) => {
                match (x, (op_b << 4) + op_l) {
                    (x, 0x07) => self.v[x as usize] = self.delay, // MOVD
                    (x, 0x0A) => { // WAITKEY
                        match window {
                            Some(window) => {
                                match chip8_get_any_key(&window) {
                                    Some(key) => self.v[x as usize] = key,
                                    None => self.pc -= 2,
                                }
                            }
                            None => return Err(Box::new(UnvavailableIOError::new(
                                        "Can't fetch inputs while running headless"
                                        ))),
                        }
                    },
                    (x, 0x15) => self.delay = self.v[x as usize], // RMOVD
                    (x, 0x18) => self.sound = self.v[x as usize], // RMOVS
                    (x, 0x1E) => { // ADDI
                        self.i = (self.i + self.v[x as usize] as u16) & 0b0000111111111111;
                        // self.v[0xF] = 1; // if u12 overflows (on amiga at least)
                    },
                    (x, 0x29) => { // LOADFNT
                        let offset = (self.v[x as usize] & 0x0F) as u16;
                        self.i = CHIP8_FONT_START + CHIP8_FONT_HEIGHT as u16 * offset;
                    },
                    (x, 0x33) => { // DCB
                        let byte = self.v[x as usize];
                        match self.ram.set_byte(self.i, byte / 100) {
                            Ok(()) => (),
                            Err(err) => return Err(err),
                        };
                        match self.ram.set_byte(self.i+1, (byte % 100) / 10) {
                            Ok(()) => (),
                            Err(err) => return Err(err),
                        };
                        match self.ram.set_byte(self.i+2, byte % 10) {
                            Ok(()) => (),
                            Err(err) => return Err(err),
                        };
                    },
                    (n, 0x55) => { // STORE
                        if (self.i + n as u16) & 0x0F > 0xFFF {
                            return Err(Box::new(InvalidAccessError::new(
                                        format!("Cannot STORE {:X} bytes of data at 0x{:03X}",
                                                n, self.i))));
                        }
                        for i in 0..=n {
                            match self.ram.set_byte(self.i + i as u16, self.v[i as usize]) {
                                Ok(()) => (),
                                Err(err) => return Err(err),
                            }
                        }
                        self.i = (self.i + n as u16 + 1) & 0b0000111111111111;
                    },
                    (n, 0x65) => { // LOAD
                        if (self.i + n as u16) & 0x0F > 0xFFF {
                            return Err(Box::new(InvalidAccessError::new(
                                        format!("Cannot LOAD {:X} bytes of data from 0x{:03X}",
                                                n, self.i))));
                        }
                        for i in 0..=n {
                            match self.ram.get(self.i + i as u16, 1) {
                                Ok(byte) => self.v[i as usize] = byte[0],
                                Err(err) => return Err(err),
                            }
                        }
                        self.i = (self.i + n as u16 + 1) & 0b0000111111111111;
                    }
                    (x, b) => return Err(Box::new(InvalidInstructionError::new(
                                format!("Wrong operand 0x{:03X} for opcode 0xF",
                                        u16::from_be_bytes([x, b]))))),
                }
            }

            err => return Err(Box::new(InvalidInstructionError::new(
                        format!("invalid opcode 0x{:X}",
                                u16::from_be_bytes([err.0 << 4 + err.1, err.2 << 4 + err.3]))
                        ))),
        };
        self.pc = (self.pc + 2) & 0b0000111111111111;
        Ok(())
    }
}
