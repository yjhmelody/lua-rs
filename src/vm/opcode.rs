//! Lua Bytecode format
//!
//! |0-5(6bits)|6-13(8bit)|14-22(9bit)|23-31(9bit)|
//! |----------|----------|-----------|-----------|
//! |  opcode  |    A     |     C     |    B      |
//!
//! |0-5(6bits)|6-13(8bit)| 14-31(18bit)|
//! |----------|----------|-------------|
//! |  opcode  |    A     | Bx(unsigned)|
//! |  opcode  |    A     | sBx(signed) |
//!
//!
//! |0-5(6bits)|6-31(26bit)|
//! |----------|-----------|
//! |  opcode  |    Ax     |
//!

/// R(A) := R(B)
pub const OP_MOVE: u8 = 0x00;
/// R(A) := Kst(Bx)
pub const OP_LOADK: u8 = 0x01;
/// R(A) := Kst(extra arg)
pub const OP_LOADKX: u8 = 0x02;
/// R(A) := (bool)B; if (C) pc++
pub const OP_LOADBOOL: u8 = 0x03;
/// R(A), R(A+1), ..., R(A+B) := nil
pub const OP_LOADNIL: u8 = 0x04;
/// R(A) := UpValue[B]
pub const OP_GETUPVAL: u8 = 0x05;
/// R(A) := UpValue[B][RK(C)]
pub const OP_GETTABUP: u8 = 0x06;
/// R(A) := R(B)[RK(C)]
pub const OP_GETTABLE: u8 = 0x07;
/// UpValue[A][RK(B)] := RK(C)
pub const OP_SETTABUP: u8 = 0x08;
/// UpValue[B] := R(A)
pub const OP_SETUPVAL: u8 = 0x09;
/// R(A)[RK(B)] := RK(C)
pub const OP_SETTABLE: u8 = 0x0a;
/// R(A) := {} (size = B,C)
pub const OP_NEWTABLE: u8 = 0x0b;
/// R(A+1) := R(B); R(A) := R(B)[RK(C)]
pub const OP_SELF: u8 = 0x0c;
/// R(A) := RK(B) + RK(C)
pub const OP_ADD: u8 = 0x0d;
/// R(A) := RK(B) - RK(C)
pub const OP_SUB: u8 = 0x0e;
/// R(A) := RK(B) * RK(C)
pub const OP_MUL: u8 = 0x0f;
/// R(A) := RK(B) % RK(C)
pub const OP_MOD: u8 = 0x10;
/// R(A) := RK(B) ^ RK(C)
pub const OP_POW: u8 = 0x11;
/// R(A) := RK(B) / RK(C)
pub const OP_DIV: u8 = 0x12;
/// R(A) := RK(B) // RK(C)
pub const OP_IDIV: u8 = 0x13;
/// R(A) := RK(B) & RK(C)
pub const OP_BAND: u8 = 0x14;
/// R(A) := RK(B) | RK(C)
pub const OP_BOR: u8 = 0x15;
/// R(A) := RK(B) ~ RK(C)
pub const OP_BXOR: u8 = 0x16;
/// R(A) := RK(B) << RK(C)
pub const OP_SHL: u8 = 0x17;
/// R(A) := RK(B) >> RK(C)
pub const OP_SHR: u8 = 0x18;
/// R(A) := -R(B)
pub const OP_UNM: u8 = 0x19;
/// R(A) := ~R(B)
pub const OP_BNOT: u8 = 0x1a;
/// R(A) := not R(B)
pub const OP_NOT: u8 = 0x1b;
/// R(A) := length of R(B)
pub const OP_LEN: u8 = 0x1c;
/// R(A) := R(B).. ... ..R(C)
pub const OP_CONCAT: u8 = 0x1d;
/// pc+=sBx; if (A) close all upvalues >= R(A - 1)
pub const OP_JMP: u8 = 0x1e;
/// if ((RK(B) == RK(C)) ~= A) then pc++
pub const OP_EQ: u8 = 0x1f;
/// if ((RK(B) <  RK(C)) ~= A) then pc++
pub const OP_LT: u8 = 0x20;
/// if ((RK(B) <= RK(C)) ~= A) then pc++
pub const OP_LE: u8 = 0x21;
/// if not (R(A) <=> C) then pc++
pub const OP_TEST: u8 = 0x22;
/// if (R(B) <=> C) then R(A) := R(B) else pc++
pub const OP_TESTSET: u8 = 0x23;
/// R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
pub const OP_CALL: u8 = 0x24;
/// return R(A)(R(A+1), ... ,R(A+B-1))
pub const OP_TAILCALL: u8 = 0x25;
/// return R(A), ... ,R(A+B-2)
pub const OP_RETURN: u8 = 0x26;
/// R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
pub const OP_FORLOOP: u8 = 0x27;
/// R(A)-=R(A+2); pc+=sBx
pub const OP_FORPREP: u8 = 0x28;
/// R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
pub const OP_TFORCALL: u8 = 0x29;
/// if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
pub const OP_TFORLOOP: u8 = 0x2a;
/// R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
pub const OP_SETLIST: u8 = 0x2b;
/// R(A) := closure(KPROTO[Bx])
pub const OP_CLOSURE: u8 = 0x2c;
/// R(A), R(A+1), ..., R(A+B-2) = vararg
pub const OP_VARARG: u8 = 0x2d;
/// extra (larger) argument for previous opcode
pub const OP_EXTRAARG: u8 = 0x2e;

/// OpMode
#[derive(Debug, Copy, Clone)]
pub enum OpMode {
    /// iABC
    ABC = 0,
    /// iABx
    ABx = 1,
    /// iAsBx
    AsBx = 2,
    /// iAx
    Ax = 3,
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

/// B, C, mode, name
pub const OPCODES: &'static [OpCode] = &[
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "MOVE    "),
    opcode(OpArgMask::K, OpArgMask::N, OpMode::ABx, "LOADK   "),
    opcode(OpArgMask::N, OpArgMask::N, OpMode::ABx, "LOADKX  "),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "LOADBOOL"),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "LOADNIL "),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "GETUPVAL"),
    opcode(OpArgMask::U, OpArgMask::K, OpMode::ABC, "GETTABUP"),
    opcode(OpArgMask::R, OpArgMask::K, OpMode::ABC, "GETTABLE"),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SETTABUP"),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "SETUPVAL"),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SETTABLE"),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "NEWTABLE"),
    opcode(OpArgMask::R, OpArgMask::K, OpMode::ABC, "SELF    "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "ADD     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SUB     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "MUL     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "MOD     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "POW     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "DIV     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "IDIV    "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BAND    "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BOR     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "BXOR    "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SHL     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "SHR     "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "UNM     "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "BNOT    "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "NOT     "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::ABC, "LEN     "),
    opcode(OpArgMask::R, OpArgMask::R, OpMode::ABC, "CONCAT  "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::AsBx, "JMP     "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "EQ      "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "LT      "),
    opcode(OpArgMask::K, OpArgMask::K, OpMode::ABC, "LE      "),
    opcode(OpArgMask::N, OpArgMask::U, OpMode::ABC, "TEST    "),
    opcode(OpArgMask::R, OpArgMask::U, OpMode::ABC, "TESTSET "),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "CALL    "),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "TAILCALL"),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "RETURN  "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::AsBx, "FORLOOP "),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::AsBx, "FORPREP "),
    opcode(OpArgMask::N, OpArgMask::U, OpMode::ABC, "TFORCALL"),
    opcode(OpArgMask::R, OpArgMask::N, OpMode::AsBx, "TFORLOOP"),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::ABC, "SETLIST "),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABx, "CLOSURE "),
    opcode(OpArgMask::U, OpArgMask::N, OpMode::ABC, "VARARG  "),
    opcode(OpArgMask::U, OpArgMask::U, OpMode::Ax, "EXTRAARG"),
];

const fn opcode(b_mode: OpArgMask, c_mode: OpArgMask, op_mode: OpMode, name: &'static str) -> OpCode {
    OpCode {
        b_mode,
        c_mode,
        op_mode,
        name,
    }
}

/// Lua Instruction, 32 bits
pub struct OpCode {
    /// B arg mode
    pub b_mode: OpArgMask,
    /// C arg mode
    pub c_mode: OpArgMask,
    /// op mode
    pub op_mode: OpMode,
    /// Op code's name
    pub name: &'static str,
}
