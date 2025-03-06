use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};
use rand::SeedableRng;
use rand::{Rng, rngs::StdRng};
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

use crate::debug::Backtrace;
use crate::mem::{Chip8Mem, Memory16Bit};
use crate::errors::{InvalidInstructionError, ProgramLoadingError, InvalidAccessError};

pub trait System {
    fn init() -> Self;
    fn load_program(&mut self, program_data: &[u8]) -> Result<(), Box<dyn Error>>;
    fn exec_instruction(&mut self, input: Arc<RwLock<WinitInputHelper>>) -> Result<(), Box<dyn Error>>;
}

pub struct Chip8 {
    i: u16,
    sp: u16,
    pc: u16,
    v: Vec<u8>,
    delay: u8,
    sound: u8,
    ram: Chip8Mem,
    rng: StdRng,
    last_frame: Instant,
    pc_backtrace: Backtrace<u16>,
}

const CHIP8_PC_START: u16 = 0x200;
const CHIP8_MAX_PROG_SIZE: u16 = CHIP8_STACK_BASE_ADDR - CHIP8_PC_START;
pub const CHIP8_DISP_BUF_ADDR: u16 = 0xF00;
pub const CHIP8_DISP_BUF_LEN: u16 = 0x100;
pub const CHIP8_DISP_WIDTH: u16 = 64;
pub const CHIP8_DISP_HEIGHT: u16 = 32;
pub const CHIP8_STACK_BASE_ADDR: u16 = 0xEA0;
pub const CHIP8_FONT_START: u16 = 0x50;
pub const CHIP8_FONT_HEIGHT: u8 = 0x5;
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
const MINIFB_COLOR_BG: [u8; 4] = [0x22, 0x11, 0x10, 0xFF];
const MINIFB_COLOR_FG: [u8; 4] = [0xFF, 0x99, 0x00, 0xFF];

pub struct Chip8State {
    pub i: u16,
    pub sp: u16,
    pub pc: u16,
    pub v: Vec<u8>,
    pub delay: u8,
    pub sound: u8,
    pub ram: Chip8Mem,
}
impl From<&Chip8> for Chip8State {
    fn from(chip8: &Chip8) -> Self {
        Self {
            i: chip8.i,
            sp: chip8.sp,
            pc: chip8.pc,
            v: chip8.v.clone(),
            delay: chip8.delay,
            sound: chip8.sound,
            ram: chip8.ram.clone()
        }
    }
}

impl Chip8 {
    pub fn get_state(&self) -> Chip8State {
        self.into()
    }

    pub fn get_mem(&self) -> &Chip8Mem {
        &self.ram
    }

    pub fn get_backtrace(&self) -> &Backtrace<u16> {
        &self.pc_backtrace
    }

    pub fn set_pixels_frame(&self, frame: &mut [u8]) {
         self.ram
             .get(CHIP8_DISP_BUF_ADDR, CHIP8_DISP_BUF_LEN)
             .unwrap()
             .into_iter()
             .enumerate()
             .for_each(|(i, c)| {
                 frame[i*32..(i+1)*32].copy_from_slice([
                     if c & 0b10000000 == 0b10000000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b01000000 == 0b01000000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00100000 == 0b00100000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00010000 == 0b00010000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00001000 == 0b00001000 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00000100 == 0b00000100 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00000010 == 0b00000010 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                     if c & 0b00000001 == 0b00000001 {MINIFB_COLOR_FG} else {MINIFB_COLOR_BG},
                 ].concat().as_slice());}
             );
    }

    fn is_key_pressed(input: Arc<RwLock<WinitInputHelper>>, key_byte: u8) -> bool {
        let key = match key_byte {
            0x0 => KeyCode::Numpad0,
            0x1 => KeyCode::Numpad1,
            0x2 => KeyCode::Numpad2,
            0x3 => KeyCode::Numpad3,
            0x4 => KeyCode::Numpad4,
            0x5 => KeyCode::Numpad5,
            0x6 => KeyCode::Numpad6,
            0x7 => KeyCode::Numpad7,
            0x8 => KeyCode::Numpad8,
            0x9 => KeyCode::Numpad9,
            0xA => KeyCode::NumpadDecimal,
            0xB => KeyCode::NumpadEnter,
            0xC => KeyCode::NumpadAdd,
            0xD => KeyCode::NumpadSubtract,
            0xE => KeyCode::NumpadMultiply,
            0xF => KeyCode::NumpadDivide,
            _ => unreachable!(),
        };
        return input.read().unwrap().key_held(key);
    }

    fn probe_keypress(input: Arc<RwLock<WinitInputHelper>>) -> Option<u8> {
        let keycodes = [
            KeyCode::Numpad0,
            KeyCode::Numpad1,
            KeyCode::Numpad2,
            KeyCode::Numpad3,
            KeyCode::Numpad4,
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
            KeyCode::Numpad9,
            KeyCode::NumpadComma,
            KeyCode::NumpadEnter,
            KeyCode::NumpadAdd,
            KeyCode::NumpadSubtract,
            KeyCode::NumpadMultiply,
            KeyCode::NumpadDivide,
        ];
        let mut key = None;
        for cur_key in keycodes {
            if let Ok(input) = input.read() {
                if input.key_pressed(cur_key) {
                    key = Some(cur_key);
                }
            }
        }
        match key {
            Some(KeyCode::Numpad0) => Some(0x0),
            Some(KeyCode::Numpad1) => Some(0x1),
            Some(KeyCode::Numpad2) => Some(0x2),
            Some(KeyCode::Numpad3) => Some(0x3),
            Some(KeyCode::Numpad4) => Some(0x4),
            Some(KeyCode::Numpad5) => Some(0x5),
            Some(KeyCode::Numpad6) => Some(0x6),
            Some(KeyCode::Numpad7) => Some(0x7),
            Some(KeyCode::Numpad8) => Some(0x8),
            Some(KeyCode::Numpad9) => Some(0x9),
            Some(KeyCode::NumpadComma) => Some(0xA),
            Some(KeyCode::NumpadEnter) => Some(0xB),
            Some(KeyCode::NumpadAdd) => Some(0xC),
            Some(KeyCode::NumpadSubtract) => Some(0xD),
            Some(KeyCode::NumpadMultiply) => Some(0xE),
            Some(KeyCode::NumpadDivide) => Some(0xF),
            Some(_) => unreachable!(),
            None => None,
        }
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
            rng: StdRng::from_os_rng(),
            last_frame: Instant::now(),
            pc_backtrace: Backtrace::new(100),
        }
    }

    fn load_program(&mut self, program_data: &[u8]) -> Result<(), Box<dyn Error>> {
        if program_data.len() > CHIP8_MAX_PROG_SIZE as usize {
            return Err(Box::new(ProgramLoadingError::new(format!(
                        "Program too long, {}B > 3,328 KB", program_data.len())
                        )))
        }
        self.ram.set(CHIP8_FONT_START, &CHIP8_FONT).unwrap();
        self.ram.set(CHIP8_PC_START, program_data).unwrap(); // shouldn't return an Err, so we
                                                             // unwrap and panic if something
                                                             // real bad happens
        Ok(())
    }

    fn exec_instruction(
        &mut self,
        input: Arc<RwLock<WinitInputHelper>>
    ) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        // TODO: better timing ? waiting for vblank on draw (check details) ?
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
                                              u16::from_be_bytes([err.0, (err.1 << 4) + err.2]))
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
                let _ = self.ram.set(self.sp, &self.pc.to_be_bytes());
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
            (0xC, x, b, l) => self.v[x as usize] = self.rng.random_range(0..=255) & ((b << 4) + l),

            // D - DISP (draws sprite @ coord VX,VY, N pixels high)
            (0xD, x, y, n) => {
                if n > 0xf { // --> FIXME: litteraly impossible, this if shouldn't exist
                    return Err(Box::new(InvalidInstructionError::new(format!("Trying to draw a sprite with height {}. Height should be between 1 and 15 both included.", n))));
                }
                let sprite = match self.ram.get(self.i, n as u16) {
                    Ok(slice) => slice.to_vec(),
                    Err(_err) => todo!(),
                };
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
                match (b, m, l) {
                    (x, 0x9, 0xE) => if Self::is_key_pressed(input, self.v[x as usize]) {
                        self.pc += 2;
                    },
                    (x, 0xA, 0x1) => if !Self::is_key_pressed(input, self.v[x as usize]) {
                        self.pc += 2;
                    },
                    err => return Err(Box::new(InvalidInstructionError::new(
                                      format!("wrong operand 0x{:03X} for opcode 0xE (should be 0xE[X]9E or 0xE[X]A1)",
                                              u16::from_be_bytes([err.0, (err.1 << 4) + err.2]))
                                      ))),
                }
            },

            // F - MISC things
            (0xF, x, op_b, op_l) => {
                match (x, (op_b << 4) + op_l) {
                    (x, 0x07) => self.v[x as usize] = self.delay, // MOVD
                    (x, 0x0A) => { // WAITKEY
                        if let Some(key) = Self::probe_keypress(input) {
                            self.v[x as usize] = key;
                        } else {
                            self.pc -= 2;
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
                        if self.i + (n as u16 & 0x0F) > 0xFFF {
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
                        if self.i + (n as u16 & 0x0F) > 0xFFF {
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
                                u16::from_be_bytes([(err.0 << 4) + err.1, (err.2 << 4) + err.3]))
                        ))),
        };
        self.pc_backtrace.refresh(self.pc, self.get_state(), opcode);
        self.pc = (self.pc + 2) & 0b0000111111111111;
        Ok(())
    }
}
