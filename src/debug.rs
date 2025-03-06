use core::fmt;
use std::fmt::{Display, LowerHex};

use crate::systems::Chip8;
use crate::mem::{Memory16Bit, Chip8Mem};

pub struct Backtrace<T> {
    trace: Box<[T]>,
    cur: usize,
}

impl<T: LowerHex + Default + Clone> Backtrace<T> {
    pub fn new(size: usize) -> Self {
        Self { trace: vec![T::default(); size].into_boxed_slice(), cur: 0}
    }
    pub fn refresh(&mut self, new_val: T) {
        self.cur = (self.cur + 1) % self.trace.len();
        self.trace[self.cur] = new_val;
    }
}

impl<T: LowerHex + Default + Clone> Display for Backtrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PC Backtrace: ")?;

        let l = self.trace.len();
        for i in 1..l {
            write!(f, "{:x}, ", self.trace[(self.cur + i) % l])?;
        }
        write!(f, "{:x}", self.trace[self.cur])
    }
}

impl Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.get_state();
        let next_instr = s.6.get(s.2, 0x2).unwrap();
        write!(f, "\x1b[1mCurrent state : \x1b[0m\n\
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
                   sound : 0x{:02x}\n",
                   s.0, s.1, s.2,
                   s.3[0], s.3[1], s.3[2],   s.3[3],   s.3[4],   s.3[5],   s.3[6],   s.3[7],
                   s.3[8], s.3[9], s.3[0xA], s.3[0xB], s.3[0xC], s.3[0xD], s.3[0xE], s.3[0xF],
                   s.4, s.5, u16::from_be_bytes([next_instr[0], next_instr[1]]))
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
