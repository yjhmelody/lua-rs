use super::opcode::*;

/// Value: 262143
const MAXARG_BX: isize = (1 << 18) - 1;
/// Value: 131071
const MAXARG_SBX: isize = MAXARG_BX >> 1;

/// Instruction decode
pub trait Instruction {
    fn opname(self) -> &'static str;
    fn opmode(self) -> OpMode;
    fn b_mode(self) -> OpArgMask;
    fn c_mode(self) -> OpArgMask;
    fn opcode(self) -> u8;
    fn abc(self) -> (isize, isize, isize);
    fn a_bx(self) -> (isize, isize);
    fn a_sbx(self) -> (isize, isize);
    fn ax(self) -> isize;
}

impl Instruction for u32 {
    fn opname(self) -> &'static str {
        OPCODES[self.opcode() as usize].name
    }

    fn opmode(self) -> OpMode {
        OPCODES[self.opcode() as usize].op_mode
    }

    fn b_mode(self) -> OpArgMask {
        OPCODES[self.opcode() as usize].b_mode
    }

    fn c_mode(self) -> OpArgMask {
        OPCODES[self.opcode() as usize].c_mode
    }

    fn opcode(self) -> u8 {
        self as u8 & 0x3F
    }

    fn abc(self) -> (isize, isize, isize) {
        let a = (self >> 6 & 0xFF) as isize;
        let c = (self >> 14 & 0x1FF) as isize;
        let b = (self >> 23 & 0x1FF) as isize;
        (a, b, c)
    }

    fn a_bx(self) -> (isize, isize) {
        let a = (self >> 6 & 0xFF) as isize;
        let bx = (self >> 14) as isize;
        (a, bx)
    }

    fn a_sbx(self) -> (isize, isize) {
        let (a, bx) = self.a_bx();
        (a, bx - MAXARG_SBX)
    }

    fn ax(self) -> isize {
        (self >> 6) as isize
    }
}
