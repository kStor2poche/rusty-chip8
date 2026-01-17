use crate::{
    mem::Memory16Bit,
    systems::{Chip8State, CHIP8_STACK_BASE_ADDR}
};

fn sprite_string(sprite: Vec<u8>) -> String {
    let mut res = String::new();
    for c in &sprite {
        res.push_str(
            String::from_iter([
                if c & 0b10000000 == 0b10000000 {'█'} else {'░'},
                if c & 0b01000000 == 0b01000000 {'█'} else {'░'},
                if c & 0b00100000 == 0b00100000 {'█'} else {'░'},
                if c & 0b00010000 == 0b00010000 {'█'} else {'░'},
                if c & 0b00001000 == 0b00001000 {'█'} else {'░'},
                if c & 0b00000100 == 0b00000100 {'█'} else {'░'},
                if c & 0b00000010 == 0b00000010 {'█'} else {'░'},
                if c & 0b00000001 == 0b00000001 {'█'} else {'░'},
                '\n',
            ]).as_str()
         );
    }
    if let Some(c) = sprite.last() {
        res.push_str(
            String::from_iter([
                if c & 0b10000000 == 0b10000000 {'█'} else {'░'},
                if c & 0b01000000 == 0b01000000 {'█'} else {'░'},
                if c & 0b00100000 == 0b00100000 {'█'} else {'░'},
                if c & 0b00010000 == 0b00010000 {'█'} else {'░'},
                if c & 0b00001000 == 0b00001000 {'█'} else {'░'},
                if c & 0b00000100 == 0b00000100 {'█'} else {'░'},
                if c & 0b00000010 == 0b00000010 {'█'} else {'░'},
                if c & 0b00000001 == 0b00000001 {'█'} else {'░'},
            ]).as_str()
        );
    }
    res
}

pub fn disas_instruction(opcode: (u8, u8, u8, u8), state: Option<Chip8State>) -> String {
    match opcode {
        // 0 - return subroutine (RTS) and display clear (CLS)
        (0x0, b, m, l) => {
            match (b, m, l) {

                (0x0, 0xE, 0x0) => { // CLS
                    "CLS".to_string()
                },

                (0x0, 0xE, 0xE) => { // RTS
                    if let Some(state) = state {
                        if state.sp < CHIP8_STACK_BASE_ADDR {
                            return "INVALID RTS from subroutine".to_string();
                        }
                        let addr_bytes = state.ram.get(state.sp, 2).expect("prolly a stack overflow going on :3");
                        format!("RTS (→ {:x})", u16::from_be_bytes([addr_bytes[0], addr_bytes[1]]))
                    } else {
                        "RTS".to_string()
                    }
                },

                _ => "INVALID".to_string(),
            }
        },

        // 1 - JMP
        (0x1, b, m, l) => {
            format!("JMP {b:x}{m:x}{l:x}")
        },

        // 2 - CALL
        (0x2, b, m, l) => {
            format!("CALL {b:x}{m:x}{l:x}")
        },

        // 3 - SKIP.EQ direct
        (0x3, x, b, l) => {
            if let Some(state) = state {
                let next = state.ram.get(state.pc + 2, 4).expect("SKIP.EQ to non-existant instructions");
                let eq = state.v[x as usize] == (b << 4)+l;
                let taken = if !eq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                let not_taken = if eq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                format!("SKIP.EQ v{x:X}, {b:x}{l:x} ({eq} → {taken:x}, avoids {not_taken:x})")
            } else {
                format!("SKIP.EQ v{x:X}, {b:x}{l:x}")
            }
        },

        // 4 - SKIP.NE direct
        (0x4, x, b, l) => {
            if let Some(state) = state {
                let next = state.ram.get(state.pc + 2, 4).expect("SKIP.EQ to non-existant instructions");
                let neq = state.v[x as usize] != (b << 4)+l;
                let taken = if !neq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                let not_taken = if neq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                format!("SKIP.NE v{x:X}, {b:x}{l:x} ({neq} → {taken:x}, avoids {not_taken:x})")
            } else {
                format!("SKIP.NE v{x:X}, {b:x}{l:x}")
            }
        },

        // 5 - SKIP.EQ register
        (0x5, x, y, 0x0) => {
            if let Some(state) = state {
                let next = state.ram.get(state.pc + 2, 4).expect("SKIP.EQ to non-existant instructions");
                let eq = state.v[x as usize] == state.v[y as usize];
                let taken = if !eq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                let not_taken = if eq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                format!("SKIP.EQ v{x:X}, v{y:X} ({eq} → {taken:x}, avoids {not_taken:x})")
            } else {
                format!("SKIP.EQ v{x:X}, v{y:X}")
            }
        },

        // 6 - SET direct
        (0x6, x, b, l) => format!("SET v{x:X}, {:x}" ,(b<<4) + l),

        // 7 - INCR direct
        (0x7, x, b, l) => format!("ADD v{x:X}, {:x}" ,(b<<4) + l),

        // 8 - Register based ops
        (0x8, x, y, op) => {
            match op {
                0x0 => format!("MOV v{x:X}, v{y:X}"), // MOV
                0x1 => format!("OR v{x:X}, v{y:X}"), // OR
                0x2 => format!("AND v{x:X}, v{y:X}"), // AND
                0x3 => format!("XOR v{x:X}, v{y:X}"), // XOR
                0x4 => { // ADD
                    if let Some(state) = state {
                        let res = state.v[x as usize].overflowing_add(state.v[y as usize]);
                        format!("ADD v{x:X}, v{y:X} → {:x}, vF = {}", res.0, res.1)
                    } else {
                        format!("ADD v{x:X}, v{y:X}")
                    }
                },
                0x5 => { // SUB
                    if let Some(state) = state {
                        let res = state.v[x as usize].overflowing_sub(state.v[y as usize]);
                        format!("SUB v{x:X}, v{y:X} → {:x}, vF = {}", res.0, res.1)
                    } else {
                        format!("SUB v{x:X}, v{y:X}")
                    }
                },
                0x6 => { // SHR
                    if let Some(state) = state {
                        let carry = state.v[y as usize] & 0x01;
                        format!("SHR v{x:X}, v{y:X} → vF = {:x}", carry)
                    } else {
                        format!("SHR v{x:X}, v{y:X}")
                    }
                },
                0x7 => { // RSUB
                    if let Some(state) = state {
                        let res = state.v[y as usize].overflowing_sub(state.v[x as usize]);
                        format!("SUB v{y:X}, v{x:X} → {:x}, vF = {}", res.0, res.1)
                    } else {
                        format!("SUB v{y:X}, v{x:X}")
                    }
                },
                0xE => { // SHL
                    if let Some(state) = state {
                        let carry = (state.v[y as usize] & 0x80) >> 7;
                        format!("SHL v{x:X}, v{y:X} → vF = {:x}", carry)
                    } else {
                        format!("SHL v{x:X}, v{y:X}")
                    }
                },

                _ => "INVALID".to_string(),
            }
        },

        // 9 - SKIP.NE register
        (0x9, x, y, 0x0) => {
            if let Some(state) = state {
                let next = state.ram.get(state.pc + 2, 4).expect("SKIP.EQ to non-existant instructions");
                let neq = state.v[x as usize] != state.v[y as usize];
                let taken = if !neq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                let not_taken = if neq {u16::from_be_bytes([next[0], next[1]])} else {u16::from_be_bytes([next[2], next[3]])};
                format!("SKIP.NEQ v{x:X}, v{y:X} ({neq} → {taken:x}, avoids {not_taken:x})")
            } else {
                format!("SKIP.NEQ v{x:X}, v{y:X}")
            }
        },

        // A - SETI
        (0xA, b, m, l) => format!("SETI {:x}", u16::from_be_bytes([b , (m << 4) + l])),

        // B - JMP relative
        (0xB, b, m, l) => {
            if let Some(state) = state {
                let next_pc = (state.v[0] as u16 + u16::from_be_bytes([b , (m << 4) + l])) & 0b0000111111111111;
                format!("JR v0, {:x} → {:x}",u16::from_be_bytes([b , (m << 4) + l]), next_pc)
            } else {
                format!("JR v0, {:x}",u16::from_be_bytes([b , (m << 4) + l]))
            }
        },

        // C - RAND (VX = rand() & BL)
        (0xC, x, b, l) => format!("RAND v{x:X} {:x}", ((b << 4) + l)),

        // D - DISP (draws sprite @ coord VX,VY, N pixels high)
        (0xD, x, y, n) => {
            if let Some(state) = state {
                let sprite = match state.ram.get(state.i, n as u16) {
                    Ok(slice) => slice.to_vec(),
                    Err(_err) => return "DRAW (invalid sprite)".to_string(),
                };
                format!("DRAW v{x:X}({:x}), v{y:X}({:x}), {n:x}\n{}", state.v[x as usize], state.v[y as usize], sprite_string(sprite))
            } else {
                format!("DRAW v{x:X}, v{y:X}, {n:x}")
            }
        },

        // E - INPT checking
        (0xE, b, m, l) => {
            match (b, m, l) {
                (x, 0x9, 0xE) => format!("PRESS v{x:X}"),
                (x, 0xA, 0x1) => format!("NPRESS v{x:X}"),
                _ => "INVALID".to_string(),
            }
        },

        // F - MISC things
        (0xF, x, op_b, op_l) => {
            match (x, (op_b << 4) + op_l) {
                (x, 0x07) => format!("GETD v{x:X}"), // MOVD
                (x, 0x0A) => format!("WAITKEY v{x:X}"), // WAITKEY
                (x, 0x15) => format!("SETD v{x:X}"), // RMOVD
                (x, 0x18) => format!("GETS v{x:X}"), // RMOVS
                (x, 0x1E) => format!("ADDI v{x:X}"), // ADDI
                (x, 0x29) => format!("LOADFNT v{x:X}"), // LOADFNT
                (x, 0x33) => format!("DCB v{x:X}"), // DCB
                (n, 0x55) => format!("STORE {n:X}"), // STORE
                (n, 0x65) => format!("LOAD {n:X}"), // LOAD
                _ => "INVALID".to_string(),
            }
        }

        _ => "INVALID".to_string(),
    }
}
