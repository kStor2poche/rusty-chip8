use core::fmt;

use crate::systems::{Chip8, CHIP8_DISP_BUF_ADDR, CHIP8_DISP_BUF_LEN};
use crate::mem::Memory16Bit;

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.get_state();
        let next_instr = s.6.get(s.2, 0x2).unwrap(); // can't get ram, apparently ??
        write!(f, "Current state : \n\
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
                   sound : 0x{:02x}\n\
                   Framebuffer : \n\
                   {22:?}",
                   s.0, s.1, s.2,
                   s.3[0], s.3[1], s.3[2],   s.3[3],   s.3[4],   s.3[5],   s.3[6],   s.3[7],
                   s.3[8], s.3[9], s.3[0xA], s.3[0xB], s.3[0xC], s.3[0xD], s.3[0xE], s.3[0xF],
                   s.4, s.5, u16::from_be_bytes([next_instr[0], next_instr[1]]),
                   s.6.get(CHIP8_DISP_BUF_ADDR, CHIP8_DISP_BUF_LEN).unwrap())
    }
}
