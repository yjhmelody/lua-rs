use crate::compiler::token::Token::OpMod;

/// instruction = 32bit(fixed length)
///
/// +---------------------------------------------+
/// |0-5(6bits)|6-13(8bit)|14-22(9bit)|23-31(9bit)|
/// |==========+==========+===========+===========|
/// |  opcode  |    A     |     C     |    B      |
/// |----------+----------+-----------+-----------|
/// |  opcode  |    A     |      Bx(unsigned)     |
/// |----------+----------+-----------+-----------|
/// |  opcode  |    A     |      sBx(signed)      |
/// +---------------------------------------------+
pub const OP_MOVE: u8 = 0x00;
pub const OP_LOADK: u8 = 0x01;
pub const OP_LOADKX: u8 = 0x02;
pub const OP_LOADBOOL: u8 = 0x03;
pub const OP_LOADNIL: u8 = 0x04;
pub const OP_GETUPVAL: u8 = 0x05;
pub const OP_GETTABUP: u8 = 0x06;
pub const OP_GETTABLE: u8 = 0x07;
pub const OP_SETTABUP: u8 = 0x08;
pub const OP_SETUPVAL: u8 = 0x09;
pub const OP_SETTABLE: u8 = 0x0a;
pub const OP_NEWTABLE: u8 = 0x0b;
pub const OP_SELF: u8 = 0x0c;
pub const OP_ADD: u8 = 0x0d;
pub const OP_SUB: u8 = 0x0e;
pub const OP_MUL: u8 = 0x0f;
pub const OP_MOD: u8 = 0x10;
pub const OP_POW: u8 = 0x11;
pub const OP_DIV: u8 = 0x12;
pub const OP_IDIV: u8 = 0x13;
pub const OP_BAND: u8 = 0x14;
pub const OP_BOR: u8 = 0x15;
pub const OP_BXOR: u8 = 0x16;
pub const OP_SHL: u8 = 0x17;
pub const OP_SHR: u8 = 0x18;
pub const OP_UNM: u8 = 0x19;
pub const OP_BNOT: u8 = 0x1a;
pub const OP_NOT: u8 = 0x1b;
pub const OP_LEN: u8 = 0x1c;
pub const OP_CONCAT: u8 = 0x1d;
pub const OP_JMP: u8 = 0x1e;
pub const OP_EQ: u8 = 0x1f;
pub const OP_LT: u8 = 0x20;
pub const OP_LE: u8 = 0x21;
pub const OP_TEST: u8 = 0x22;
pub const OP_TESTSET: u8 = 0x23;
pub const OP_CALL: u8 = 0x24;
pub const OP_TAILCALL: u8 = 0x25;
pub const OP_RETURN: u8 = 0x26;
pub const OP_FORLOOP: u8 = 0x27;
pub const OP_FORPREP: u8 = 0x28;
pub const OP_TFORCALL: u8 = 0x29;
pub const OP_TFORLOOP: u8 = 0x2a;
pub const OP_SETLIST: u8 = 0x2b;
pub const OP_CLOSURE: u8 = 0x2c;
pub const OP_VARARG: u8 = 0x2d;
pub const OP_EXTRAARG: u8 = 0x2e;

/// OpMode
#[derive(Debug, Copy, Clone)]
pub enum OpMode {
    /// iABC
    ABC = 0,
    /// iABx
    ABX = 1,
    /// iAsBx
    ASBX = 2,
    /// iAx
    AX = 3,
}


/// OpArgMask
#[derive(Debug, Copy, Clone)]
pub enum OpArgMask {
    /// OpArgN
    N = 0,
    /// OpArgU
    U = 1,
    /// OpArgR
    R = 2,
    /// OpArgK
    K = 3,
}

/*       B       C     mode    name    */
pub const OPCODES: &'static [OpCode] = &[
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "MOVE    "), // R(A) := R(B)
    opcode(OpArgMask::K, OpArgMask::N, OpMode::ABX, "LOADK   "), // R(A) := Kst(Bx)
    opcode(OpArgMask::N, OpArgMask::N, OpMode::ABX, "LOADKX  "), // R(A) := Kst(extra arg)
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "LOADBOOL"), // R(A) := (bool)B; if (C) pc++
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "LOADNIL "), // R(A), R(A+1), ..., R(A+B) := nil
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "GETUPVAL"), // R(A) := UpValue[B]
    opcode(OpArgMask::U, OpArgMask::K, OpMode::ABC, "GETTABUP"), // R(A) := UpValue[B][RK(C)]
    opcode(OpArgMask::R, OpArgMask::K, OpMode::ABC, "GETTABLE"), // R(A) := R(B)[RK(C)]
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SETTABUP"), // UpValue[A][RK(B)] := RK(C)
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "SETUPVAL"), // UpValue[B] := R(A)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SETTABLE"), // R(A)[RK(B)] := RK(C)
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "NEWTABLE"), // R(A) := {} (size = B,C)
    opcode(OpArgMask::R, OpArgMask::K, OpMode::ABC, "SELF    "), // R(A+1) := R(B); R(A) := R(B)[RK(C)]
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "ADD     "), // R(A) := RK(B) + RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SUB     "), // R(A) := RK(B) - RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "MUL     "), // R(A) := RK(B) * RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "MOD     "), // R(A) := RK(B) % RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "POW     "), // R(A) := RK(B) ^ RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "DIV     "), // R(A) := RK(B) / RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "IDIV    "), // R(A) := RK(B) // RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BAND    "), // R(A) := RK(B) & RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BOR     "), // R(A) := RK(B) | RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BXOR    "), // R(A) := RK(B) ~ RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SHL     "), // R(A) := RK(B) << RK(C)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SHR     "), // R(A) := RK(B) >> RK(C)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "UNM     "), // R(A) := -R(B)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "BNOT    "), // R(A) := ~R(B)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "NOT     "), // R(A) := not R(B)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "LEN     "), // R(A) := length of R(B)
    opcode(OpArgMask::R, OpArgMask::R, OpMode::ABC, "CONCAT  "), // R(A) := R(B).. ... ..R(C)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ASBX, "JMP     "), // pc+=sBx; if (A) close all upvalues >= R(A - 1)
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "EQ      "), // if ((RK(B) == RK(C)) ~= A) then pc++
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "LT      "), // if ((RK(B) <  RK(C)) ~= A) then pc++
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "LE      "), // if ((RK(B) <= RK(C)) ~= A) then pc++
    opcode(OpArgMask::N, OpArgMask::U, OpMode::ABC, "TEST    "), // if not (R(A) <=> C) then pc++
    opcode(OpArgMask::R, OpArgMask::U, OpMode::ABC, "TESTSET "), // if (R(B) <=> C) then R(A) := R(B) else pc++
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "CALL    "), // R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "TAILCALL"), // return R(A)(R(A+1), ... ,R(A+B-1))
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "RETURN  "), // return R(A), ... ,R(A+B-2)
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ASBX, "FORLOOP "), // R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ASBX, "FORPREP "), // R(A)-=R(A+2); pc+=sBx
    opcode(OpArgMask::N, OpArgMask::U, OpMode::ABC, "TFORCALL"),  // R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ASBX, "TFORLOOP"), // if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "SETLIST "),  // R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABX, "CLOSURE "),  // R(A) := closure(KPROTO[Bx])
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "VARARG  "),  // R(A), R(A+1), ..., R(A+B-2) = vararg
    opcode(OpArgMask::U, OpArgMask::U, OpMode::AX, "EXTRAARG"),   // extra (larger) argument for previous opcode
];

const fn opcode(bmode: OpArgMask, cmode: OpArgMask, opmode: OpMode, name: &'static str) -> OpCode {
    OpCode {
        bmode,
        cmode,
        opmode,
        name,
    }
}

pub struct OpCode {
    /// B arg mode
    pub bmode: OpArgMask,
    /// C arg mode
    pub cmode: OpArgMask,
    /// op mode
    pub opmode: OpMode,
    /// Op code's name
    pub name: &'static str,
}
