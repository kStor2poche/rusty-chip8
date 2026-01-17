use core::fmt;
use std::fmt::{Display, LowerHex};

use crate::disas::disas_instruction;
use crate::systems::Chip8State;
use crate::mem::{Memory16Bit, Chip8Mem};

pub struct Backtrace<T> {
    trace: Box<[(T, String)]>,
    cur: usize,
}

impl<T: LowerHex + Default + Clone> Backtrace<T> {
    pub fn new(size: usize) -> Self {
        Self { trace: vec![(T::default(), String::new()); size].into_boxed_slice(), cur: 0}
    }
    pub fn refresh(&mut self, new_val: T, state: Chip8State, cur_op: (u8, u8, u8, u8)) {
        self.cur = (self.cur + 1) % self.trace.len();
        self.trace[self.cur] = (new_val, disas_instruction(cur_op, Some(state)));
    }
}

impl<T: LowerHex + Default + Clone> Display for Backtrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\x1b[1mBacktrace:\x1b[0m")?;

        let l = self.trace.len();
        for i in 1..l {
            let (addr, instr) = &self.trace[(self.cur + i) % l];
            writeln!(f, "{:x}: {}", addr, instr)?;
        }
        let (addr, instr) = &self.trace[self.cur];
        write!(f, "{:x}: {}", addr, instr)
    }
}

impl Display for Chip8State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self;
        let next_instr = s.ram.get(s.pc, 0x2).unwrap();
        writeln!(f, "\x1b[1mCurrent state : \x1b[0m\n\
                   I : 0x{:03x}  \
                   SP : 0x{:03x}  \
                   PC : 0x{:03x} -> 0x{21:04x} (next instruction)\n\
                   V0 : 0x{:02x}  \
                   V1 : 0x{:02x}  \
                   V2 : 0x{:02x}  \
                   V3 : 0x{:02x}\n\
                   V4 : 0x{:02x}  \
                   V5 : 0x{:02x}  \
                   V6 : 0x{:02x}  \
                   V7 : 0x{:02x}\n\
                   V8 : 0x{:02x}  \
                   V9 : 0x{:02x}  \
                   VA : 0x{:02x}  \
                   VB : 0x{:02x}\n\
                   VC : 0x{:02x}  \
                   VD : 0x{:02x}  \
                   VE : 0x{:02x}  \
                   VF : 0x{:02x}\n\
                   delay : 0x{:02x}  \
                   sound : 0x{:02x}",
                   s.i, s.sp, s.pc,
                   s.v[0], s.v[1], s.v[2],   s.v[3],   s.v[4],   s.v[5],   s.v[6],   s.v[7],
                   s.v[8], s.v[9], s.v[0xA], s.v[0xB], s.v[0xC], s.v[0xD], s.v[0xE], s.v[0xF],
                   s.delay, s.sound, u16::from_be_bytes([next_instr[0], next_instr[1]]))
    }
}

impl Display for Chip8Mem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = writeln!(f, "\x1b[1mRAM dump :\x1b[0m");
        self.dump().chunks_exact(0x10).enumerate().for_each(|(i,chnk)| {
            let _ = writeln!(f, "\x1b[37m0x{:03X} :\x1b[0m {:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:02X}{:02X} \
                                                           {:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:02X}{:02X}",
                           i*0x10,
                           chnk[0x0], chnk[0x1], chnk[0x2], chnk[0x3], chnk[0x4], chnk[0x5], chnk[0x6], chnk[0x7],
                           chnk[0x8], chnk[0x9], chnk[0xA], chnk[0xB], chnk[0xC], chnk[0xD], chnk[0xE], chnk[0xF]);
        });
        Ok(())
    }
}
