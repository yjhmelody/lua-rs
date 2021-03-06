//! ## Instruction Notation
//!
//! R(A): Register A (specified in instruction field A)
//!
//! R(B): Register B (specified in instruction field B)
//!
//! R(C): Register C (specified in instruction field C)
//!
//! PC: Program Counter
//!
//! Kst(n): Element n in the constant list
//!
//! Upvalue[n]: Name of upvalue with index n
//!
//! Gbl[sym]: Global variable indexed by symbol sym
//!
//! RK(B): Register B or a constant index
//!
//! RK(C): Register C or a constant index
//!
//! sBx: Signed displacement (in field sBx) for all kinds of jumps
//!
//! ## Instruction Summary
//!
//! Lua bytecode instructions are 32-bits in size. All instructions have an opcode in the first 6 bits. Instructions can have the following fields:
//!
//! - 'A' : 8 bits
//! - 'B' : 9 bits
//! - 'C' : 9 bits
//! - 'Ax' : 26 bits ('A', 'B', and 'C' together)
//! - 'Bx' : 18 bits ('B' and 'C' together)
//! - 'sBx' : signed Bx

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_snake_case)]

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

use crate::binary::chunk::*;
use crate::compiler::ast::*;
use crate::compiler::error::*;
use crate::compiler::lexer::Line;
use crate::compiler::token::Token;
use crate::number::parser::int_to_float_byte;
use crate::vm::opcode;

/// 262143
const MAXARG_BX: isize = (1 << 18) - 1;
/// 131071
const MAXARG_SBX: isize = MAXARG_BX >> 1;


pub fn gen_prototype(block: Box<Block>) -> Result<Rc<Prototype>> {
    let last_line = block.last_line;
    let fn_def = FnDef::new(ParList::default(), block, 0, last_line);
    let mut fn_info = FnInfo::new(None, ParList::default(), 0, last_line);
//    fn_info.add_local_var("_ENV".to_string(), 0)?;
    fn_info.up_values.insert(
        "_ENV".to_string(),
        UpValueInfo::new(Some(0), Some(0), 0),
    );


    let fn_info_ref = FnInfoRef(Rc::new(RefCell::new(fn_info)));
    FnInfo::codegen_fn_def_exp2(&fn_info_ref, &fn_def, 0)?;
    fn_info_ref.0.borrow_mut().emit_return(last_line, 0, 0);
    let proto = fn_info_ref.0.borrow_mut().to_prototype();
    Ok(proto)

//    fn_info.codegen_fn_def_exp(&fn_def, 0)?;
//    fn_info.emit_return(last_line, 0, 0);
//    Ok(fn_info.to_prototype())
}

/// Local Variable Information Reference
pub type LocalVarInfoRef = Rc<RefCell<LocalVarInfo>>;

/// Local Variable Information
#[derive(Debug, Copy, Clone)]
pub struct LocalVarInfo {
    scope_level: isize,
    slot: usize,
    is_captured: bool,
    start_pc: usize,
    end_pc: usize,
}

/// Up Value Information
#[derive(Debug, Copy, Clone)]
struct UpValueInfo {
    local_var_slot: Option<usize>,
    up_value_index: Option<usize>,
    /// The sequent of UpValue in Foreign Function
    index: usize,
}

impl UpValueInfo {
    pub fn new(local_var_slot: Option<usize>, up_value_index: Option<usize>, index: usize) -> Self {
        Self {
            local_var_slot,
            up_value_index,
            index,
        }
    }
}

/// Function Information Table Reference
#[derive(Debug, Clone)]
pub struct FnInfoRef(Rc<RefCell<FnInfo>>);

/// Function Information Table for Lua
#[derive(Debug, Clone)]
pub struct FnInfo {
    constants: HashMap<Constant, usize>,
    /// Num of used regs
    used_regs: usize,
    /// Maximum need of num of regs
    max_regs: usize,
    /// Block scope level
    scope_level: isize,
    /// Local variable of all scope
    local_vars: Vec<HashMap<String, LocalVarInfoRef>>,
    /// Record some breaks statements
    breaks: Vec<Option<Vec<usize>>>,
    /// Parents' index
//    parent_index: usize,
    /// 当前节点的索引
//    current_index: usize,
    /// UpValues
    up_values: HashMap<String, UpValueInfo>,
    /// Store Lua instructions
    instructions: Vec<u32>,
    /// Nested Functions
    sub_fns: Vec<FnInfoRef>,
    /// parent function
    parent: Option<FnInfoRef>,
    /// The function's param num
    num_params: usize,
    /// Has `...`
    is_vararg: bool,
    /// For debug
    line_nums: Vec<u32>,
    /// For debug
    line: Line,
    /// For debug
    last_line: Line,
}

static mut COUNT: usize = 0;

fn count() -> usize {
    unsafe {
        COUNT += 1;
        COUNT
    }
}

impl FnInfoRef {
    #[inline]
    fn new(parent: Option<FnInfoRef>, par_list: ParList, line: Line, last_line: Line) -> Self {
        let is_vararg = par_list.is_vararg;
        let num_params = par_list.params.len();

        Self(Rc::new(RefCell::new(FnInfo::new(
            parent,
            par_list,
            line,
            last_line,
        ))))
    }
}

impl Default for FnInfoRef {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(FnInfo::default())))
    }
}


/********************** keep function information ************************/

impl Default for FnInfo {
    fn default() -> Self {
        Self::new(None, ParList::default(), 0, 0)
    }
}

impl FnInfo {
    /// Create a FnInfo structure
    #[inline]
    pub fn new(parent: Option<FnInfoRef>, par_list: ParList, line: Line, last_line: Line) -> Self {
        let is_vararg = par_list.is_vararg;
        let num_params = par_list.params.len();
//        let parent_index = 0;
//        let current_index = 0;
        Self {
            constants: HashMap::new(),
            used_regs: 0,
            max_regs: 0,
            scope_level: 0,
            local_vars: vec![HashMap::new()],
            breaks: vec![None],
//            parent_index,
//            current_index,
            up_values: HashMap::new(),
            instructions: Vec::new(),
            sub_fns: Vec::new(),
            parent,
            num_params,
            is_vararg,
            line_nums: Vec::new(),
            line,
            last_line,
        }
    }


    fn constant_index(&mut self, k: &Constant) -> usize {
        match self.constants.get(k) {
            Some(v) => *v,
            None => {
                let idx = self.constants.len();
                self.constants.insert(k.clone(), idx);
                idx
            }
        }
    }

    fn alloc_register(&mut self) -> Result<usize> {
        self.used_regs += 1;
        if self.used_regs >= 255 {
            return Err(Error::NoMoreRegisters);
        } else if self.used_regs > self.max_regs {
            self.max_regs = self.used_regs;
        }
        Ok(self.used_regs - 1)
    }

    fn alloc_registers(&mut self, n: usize) -> Result<usize> {
        for _ in 0..n {
            self.alloc_register()?;
        }
        Ok(self.used_regs - n)
    }

    #[inline]
    fn free_register(&mut self) {
        assert_ne!(self.used_regs, 0);
        self.used_regs -= 1;
    }

    #[inline]
    fn free_registers(&mut self, n: usize) {
        for _ in 0..n {
            self.free_register();
        }
    }

    #[inline]
    fn get_current_scope(&self) -> &HashMap<String, LocalVarInfoRef> {
        &self.local_vars[self.scope_level as usize]
    }

    #[inline]
    fn get_current_scope_mut(&mut self) -> &mut HashMap<String, LocalVarInfoRef> {
        &mut self.local_vars[self.scope_level as usize]
    }

    /// Create a new scope for vars, true for breakable
    #[inline]
    fn enter_scope(&mut self, breakable: bool) {
        self.scope_level += 1;
        self.local_vars.push(HashMap::new());
        if breakable {
            self.breaks.push(Some(vec![]));
        } else {
            self.breaks.push(None);
        }
    }

    /// Exit current scope
    #[inline]
    fn exit_scope(&mut self) -> Result<()> {
        let pending_break_jmps = self.breaks.pop().ok_or(Error::NoMoreScopes)?;
        let a = self.get_jump_arg_a() as usize;

        if let Some(pending_break_jmps) = pending_break_jmps {
            for pc in pending_break_jmps {
                let sBx = self.pc() - pc;
                let i = (sBx + MAXARG_SBX as usize) << 14 | a << 6 | opcode::OP_JMP as usize;
                self.instructions[pc] = i as u32;
            }
        }

        self.scope_level -= 1;
        match self.local_vars.pop() {
            Some(vars) => Ok(()),
            None => Err(Error::NoMoreScopes)
        }
    }

    fn get_jump_arg_a(&mut self) -> isize {
        let mut has_captured_local_var = false;
        let mut min_local_var_slot = self.max_regs;
        // fixme
        let local_vars = self.get_current_scope_mut();
        local_vars.iter().for_each(|(k, mut local_var)| {
            if local_var.borrow_mut().is_captured {
                has_captured_local_var = true;
            }
            if local_var.borrow_mut().slot < min_local_var_slot && k.starts_with('(') {
                min_local_var_slot = local_var.borrow_mut().slot;
            }
        });

        if has_captured_local_var {
            min_local_var_slot as isize + 1
        } else {
            0
        }
    }

    /// Add a local variable and return register index
    fn add_local_var(&mut self, name: String, start_pc: usize) -> Result<usize> {
        let mut new_var = Rc::new(RefCell::new(LocalVarInfo {
            scope_level: self.scope_level,
            slot: self.alloc_register()?,
            is_captured: false,
            start_pc,
            end_pc: 0,
        }));
        let slot = new_var.borrow_mut().slot;
        if self.local_vars.len() as isize <= self.scope_level {
            self.local_vars.resize(self.local_vars.len() * 2, HashMap::new());
        }

        self.local_vars[self.scope_level as usize].insert(name, new_var);
        Ok(slot)
    }

    /// Get name's register number
    fn local_var_slot(&self, name: &String) -> Result<usize> {
        for local_var in self.local_vars.iter() {
            match local_var.get(name) {
                Some(mut local_var) => { return Ok(local_var.borrow_mut().slot) },
                _ => {},
            }
        }
        Err(Error::IllegalRegister)
    }

    /// Remove a variable from current scope
    #[inline]
    fn remove_local_var(&mut self, name: &String) {
        self.free_register();
        self.get_current_scope_mut().remove(name);
    }

    /// Create a jump instruction to a latest loop block
    fn add_break_jump(&mut self, pc: usize, line: Line) -> Result<()> {
        for brk in &mut self.breaks.iter_mut().rev() {
            match brk.as_mut() {
                Some(arr) => {
                    arr.push(pc);
                    return Ok(());
                }
                None => {}
            }
        }
        Err(Error::NoLoop { line })
    }

    /// Get up value's index
    fn up_value_index(&mut self, name: &String) -> Option<usize> {
        // 找出如 _ENV 的upvalue 会一直在外围函数查询
        // todo: refactor scope lookup
        if let Some(up_value) = self.up_values.get(name) {
            return Some(up_value.index);
            // fixme
        } else if let Some(ref mut parent) = self.parent {
            match parent.0.borrow_mut().get_local_var(name) {
                Some(local_var) => {
                    let idx = self.up_values.len();
                    self.up_values.insert(name.clone(), UpValueInfo::new(Some(local_var.borrow_mut().slot), None, idx));
                    local_var.borrow_mut().is_captured = true;
                    return Some(idx);
                }

                None => {}
            }
            match parent.0.borrow_mut().up_value_index(name) {
                Some(upval_idx) => {
                    let idx = self.up_values.len();
                    self.up_values.insert(name.clone(), UpValueInfo::new(None, Some(upval_idx), idx));
                    return Some(idx);
                }
                None => {}
            }
        }

        None
    }

    fn get_local_var(&mut self, name: &String) -> Option<&mut LocalVarInfoRef> {
        for map in self.local_vars.iter_mut() {
            match map.get_mut(name) {
                Some(local_var) => { return Some(local_var); }
                None => {}
            }
        }
        None
    }

    fn close_open_up_values(&mut self, line: Line) {
        // todo: understand
        let a = self.get_jump_arg_a();
        if a > 0 {
            self.emit_jmp(line, a, 0);
        }
    }

    fn to_prototype(&self) -> Rc<Prototype> {
        Rc::new(Prototype {
            source: None,
            line_defined: self.line as u32,
            last_line_defined: self.last_line as u32,
            num_params: self.num_params as u8,
            is_vararg: self.is_vararg as u8,
            max_stack_size: self.max_regs as u8,
            code: self.instructions.clone(),
            constants: self.get_constants(),
            up_values: self.get_up_values(),
            prototypes: Self::to_prototypes(&self.sub_fns),
            line_info: self.line_nums.clone(),
            local_vars: self.get_local_vars(),
            up_value_names: self.get_up_value_names(),
        })
    }

    fn get_local_vars(&self) -> Vec<LocalVar> {
        let mut res = vec![];
        // todo: maybe refactor it to linear structure
        for local_var in self.local_vars.iter() {
            for (name, loc_var) in local_var.iter() {
                let start_pc = loc_var.borrow_mut().start_pc as u32;
                let end_pc = loc_var.borrow_mut().end_pc as u32;
                res.push(LocalVar {
                    var_name: name.clone(),
                    start_pc,
                    end_pc,
                })
            }
        }

        res
    }

    fn get_up_value_names(&self) -> Vec<String> {
        let mut names: Vec<_> = iter::repeat(String::default()).take(self.up_values.len()).collect();
        for (name, up_val) in self.up_values.iter() {
            names[up_val.index] = name.clone();
        }
        names
    }

    fn to_prototypes(sub_fns: &Vec<FnInfoRef>) -> Vec<Rc<Prototype>> {
        sub_fns.iter().map(|sub_fn| {
            sub_fn.0.borrow_mut().to_prototype()
        }).collect()
    }

    fn get_constants(&self) -> Vec<Constant> {
        let mut consts: Vec<Constant> = iter::repeat(Constant::Nil).take(self.constants.len()).collect();
        self.constants.iter().for_each(|(cst, &index)| {
            consts[index] = cst.clone();
        });
        consts
    }

    fn get_up_values(&self) -> Vec<UpValue> {
        let mut up_vals: Vec<UpValue> = iter::repeat(UpValue::default()).take(self.up_values.len()).collect();

        self.up_values.iter().for_each(|(_, up_val)| {
            // instack
            // fixme
            if up_val.local_var_slot.is_some() {
                up_vals[up_val.index] = UpValue::new(1, up_val.local_var_slot.unwrap() as u8);
            } else {
                up_vals[up_val.index] = UpValue::new(0, up_val.up_value_index.unwrap() as u8);
            }
        });

        up_vals
    }
}

/********************** emit bytecode ************************/

impl FnInfo {
    #[inline]
    fn emit_ABC(&mut self, line: Line, opcode: u8, a: isize, b: isize, c: isize) {
        let ins = b << 23 | c << 14 | a << 6 | opcode as isize;
        self.instructions.push(ins as u32);
        self.line_nums.push(line as u32);
    }

    #[inline]
    fn emit_ABx(&mut self, line: Line, opcode: u8, a: isize, bx: isize) {
        let ins = bx << 14 | a << 6 | opcode as isize;
        self.instructions.push(ins as u32);
        self.line_nums.push(line as u32);
    }

    #[inline]
    fn emit_AsBx(&mut self, line: Line, opcode: u8, a: isize, b: isize) {
        let ins = (b + MAXARG_SBX) << 14 | a << 6 | opcode as isize;
        self.instructions.push(ins as u32);
        self.line_nums.push(line as u32);
    }

    #[inline]
    fn emit_Ax(&mut self, line: Line, opcode: u8, ax: isize) {
        let ins = ax << 6 | opcode as isize;
        self.instructions.push(ins as u32);
        self.line_nums.push(line as u32);
    }

    // r[a] = r[b]
    #[inline]
    fn emit_move(&mut self, line: Line, a: isize, b: isize) {
        self.emit_ABC(line, opcode::OP_MOVE, a, b, 0);
    }

    // r[a], r[a+1], ..., r[a+b] = nil
    #[inline]
    fn emit_load_nil(&mut self, line: Line, a: isize, b: isize) {
        self.emit_ABC(line, opcode::OP_LOADNIL, a, b - 1, 0);
    }

    // r[a] = b; if (c) pc++
    #[inline]
    fn emit_load_bool(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_LOADBOOL, a, b, c);
    }

    // r[a] = kst[bx]
    #[inline]
    fn emit_load_k(&mut self, line: Line, a: isize, k: Constant) {
        let idx = self.constant_index(&k) as isize;
        if idx < (1 << 18) {
            self.emit_ABx(line, opcode::OP_LOADK, a, idx);
        } else {
            self.emit_ABx(line, opcode::OP_LOADKX, a, 0);
            self.emit_Ax(line, opcode::OP_EXTRAARG, idx);
        }
    }
    // r[a], r[a+1], ..., r[a+b-2] = vararg
    #[inline]
    fn emit_vararg(&mut self, line: Line, a: isize, n: isize) {
        self.emit_ABC(line, opcode::OP_VARARG, a, n + 1, 0)
    }

    // r[a] = emit_closure(proto[bx])
    #[inline]
    fn emit_closure(&mut self, line: Line, a: isize, bx: isize) {
        self.emit_ABx(line, opcode::OP_CLOSURE, a, bx);
    }

    // r[a] = {}
    #[inline]
    fn emit_new_table(&mut self, line: Line, a: isize, n_arr: isize, n_rec: isize) {
        self.emit_ABC(line, opcode::OP_NEWTABLE, a, int_to_float_byte(n_arr), int_to_float_byte(n_rec));
    }

    // r[a][(c-1)*FPF+i] := r[a+i], 1 <= i <= b
    #[inline]
    fn emit_set_list(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_SETLIST, a, b, c);
    }

    // r[a] := r[b][rk(c)]
    #[inline]
    fn emit_get_table(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_GETTABLE, a, b, c);
    }

    // r[a][rk(b)] = rk(c)
    #[inline]
    fn emit_set_table(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_SETTABLE, a, b, c);
    }

    // r[a] = upval[b]
    #[inline]
    fn emit_get_up_value(&mut self, line: Line, a: isize, b: isize) {
        self.emit_ABC(line, opcode::OP_GETUPVAL, a, b, 0);
    }

    // upval[b] = r[a]
    #[inline]
    fn emit_set_up_value(&mut self, line: Line, a: isize, b: isize) {
        self.emit_ABC(line, opcode::OP_SETUPVAL, a, b, 0);
    }

    // r[a] = upval[b][rk(c)]
    #[inline]
    fn emit_get_table_up(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_GETTABUP, a, b, c);
    }

    // upval[a][rk(b)] = rk(c)
    #[inline]
    fn emit_set_table_up(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_SETTABUP, a, b, c);
    }

    // r[a], ..., r[a+c-2] = r[a](r[a+1], ..., r[a+b-1])
    #[inline]
    fn emit_call(&mut self, line: Line, a: isize, arg_num: isize, ret_num: isize) {
        self.emit_ABC(line, opcode::OP_CALL, a, arg_num + 1, ret_num + 1);
    }

    // return r[a](r[a+1], ... ,r[a+b-1])
    #[inline]
    fn emit_tail_call(&mut self, line: Line, a: isize, arg_num: isize) {
        self.emit_ABC(line, opcode::OP_TAILCALL, a, arg_num + 1, 0);
    }

    // return r[a], ... , r[a+b-1]
    #[inline]
    fn emit_return(&mut self, line: Line, a: isize, n: isize) {
        self.emit_ABC(line, opcode::OP_RETURN, a, n + 1, 0);
    }

    // r[a+1] := r[b]; r[a] := r[b][rk(c)]
    #[inline]
    fn emit_self(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_SELF, a, b, c);
    }

    // pc += sBx; if (a) close all upvalues >= r[a-1]
    #[inline]
    fn emit_jmp(&mut self, line: Line, a: isize, sBx: isize) -> usize {
        self.emit_AsBx(line, opcode::OP_JMP, a, sBx);
        self.instructions.len() - 1
    }

    // if not (r[a] <==> c) then pc++
    #[inline]
    fn emit_test(&mut self, line: Line, a: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_TEST, a, 0, c);
    }

    // if (r[b] <==> c) then r[a] := r[b] else pc++
    #[inline]
    fn emit_test_set(&mut self, line: Line, a: isize, b: isize, c: isize) {
        self.emit_ABC(line, opcode::OP_TEST, a, b, c);
    }

    // R(A)-=R(A+2); pc+=sBx
    #[inline]
    fn emit_for_prep(&mut self, line: Line, a: isize, sBx: isize) -> isize {
        self.emit_AsBx(line, opcode::OP_FORPREP, a, sBx);
        self.instructions.len() as isize - 1
    }

    // R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    #[inline]
    fn emit_for_loop(&mut self, line: Line, a: isize, sBx: isize) -> isize {
        self.emit_AsBx(line, opcode::OP_FORLOOP, a, sBx);
        self.instructions.len() as isize - 1
    }

    // r(a+3), ... ,r(a+2+c) := r(a)(r(a+1), r(a+2));
    #[inline]
    fn emit_t_for_call(&mut self, line: Line, a: isize, c: isize) {
        // todo: understand it
        self.emit_ABC(line, opcode::OP_TFORCALL, a, 0, c);
    }

    // if r(a+1) ~= nil then { r(a) = r(a+1); pc += sBx }
    #[inline]
    fn emit_t_for_loop(&mut self, line: Line, a: isize, sBx: isize) {
        self.emit_AsBx(line, opcode::OP_TFORLOOP, a, sBx);
    }

    // r[a] = op r[b]
    #[inline]
    fn emit_unary_op(&mut self, line: Line, op: &Token, a: isize, b: isize) {
        match op {
            Token::OpNot => self.emit_ABC(line, opcode::OP_NOT, a, b, 0),
            Token::OpWave => self.emit_ABC(line, opcode::OP_BNOT, a, b, 0),
            Token::OpLen => self.emit_ABC(line, opcode::OP_LEN, a, b, 0),
            Token::OpMinus => self.emit_ABC(line, opcode::OP_UNM, a, b, 0),
            _ => unreachable!()
        }
    }

    // r[a] = rk[b] op rk[c]
    fn emit_binary_op(&mut self, line: Line, op: &Token, a: isize, b: isize, c: isize) {
        // arith & bitwise & relational
        match op {
            Token::OpAdd => self.emit_ABC(line, opcode::OP_ADD, a, b, c),
            Token::OpMinus => self.emit_ABC(line, opcode::OP_SUB, a, b, c),
            Token::OpMul => self.emit_ABC(line, opcode::OP_MUL, a, b, c),
            Token::OpMod => self.emit_ABC(line, opcode::OP_MOD, a, b, c),
            Token::OpPow => self.emit_ABC(line, opcode::OP_POW, a, b, c),
            Token::OpDiv => self.emit_ABC(line, opcode::OP_DIV, a, b, c),
            Token::OpIDiv => self.emit_ABC(line, opcode::OP_IDIV, a, b, c),
            Token::OpBitOr => self.emit_ABC(line, opcode::OP_BOR, a, b, c),
            Token::OpBitAnd => self.emit_ABC(line, opcode::OP_BAND, a, b, c),
            Token::OpWave => self.emit_ABC(line, opcode::OP_BXOR, a, b, c),
            Token::OpShl => self.emit_ABC(line, opcode::OP_SHL, a, b, c),
            Token::OpShr => self.emit_ABC(line, opcode::OP_SHR, a, b, c),
            // relational ops
            op => {
                match op {
                    Token::OPEq => self.emit_ABC(line, opcode::OP_EQ, 1, b, c),
                    Token::OpNe => self.emit_ABC(line, opcode::OP_EQ, 0, b, c),
                    Token::OpLt => self.emit_ABC(line, opcode::OP_LT, 1, b, c),
                    Token::OpGt => self.emit_ABC(line, opcode::OP_LT, 1, c, b),
                    Token::OpLe => self.emit_ABC(line, opcode::OP_LE, 1, b, c),
                    Token::OpGe => self.emit_ABC(line, opcode::OP_LE, 1, c, b),
                    _ => unreachable!()
                }
                self.emit_jmp(line, 0, 1);
                self.emit_load_bool(line, a, 0, 1);
                self.emit_load_bool(line, a, 1, 0);
            }
        }
    }

    // return current pc
    #[inline]
    fn pc(&self) -> usize {
        self.instructions.len() - 1
    }

    // fix sbx for one instruction
    fn fix_sbx(&mut self, pc: usize, sBx: isize) {
        let mut ins = self.instructions[pc];
        // clear sBx Op
        ins = ins << 18 >> 18;
        // reset sBx op
        ins = ins | ((sBx + MAXARG_SBX) as u32) << 14;
        self.instructions[pc] = ins;
    }
}

/********************** statement code generation ************************/

impl FnInfo {
    fn codegen_block(&mut self, block: &Block) -> Result<()> {
        for stat in &block.stats {
            self.codegen_stat(stat)?;
        }
        match &block.ret_exps {
            Some(ret_exps) => {
                self.codegen_ret_stat(ret_exps, block.last_line)
            }
            None => { Ok(()) }
        }
    }

    fn codegen_stat(&mut self, stat: &Stat) -> Result<()> {
        match stat {
            Stat::FnCall(fn_call) => self.codegen_fn_call_stat(fn_call),
            Stat::Break(line) => self.codegen_break_stat(*line),
            Stat::Do(block) => self.codegen_do_stat(&*block),
            Stat::Repeat(exp, block) => self.codegen_repeat_stat(exp, &*block),
            Stat::While(exp, block) => self.codegen_while_stat(exp, &*block),
            Stat::Condition(exps, blocks) => self.codegen_condition_stat(exps, blocks),
            Stat::ForNum(for_num) => self.codegen_for_num_stat(&*for_num),
            Stat::ForIn(for_in, line) => self.codegen_for_in_stat(&*for_in, *line),
            Stat::Assign(names, vals, line) => self.codegen_assign_stat(names, vals, *line),
            Stat::LocalVarDecl(names, exps, line) => self.codegen_local_var_decl_stat(names, exps, *line),
            Stat::LocalFnDef(name, fn_def) => self.codegen_local_fn_def_stat(name, fn_def),

            _ => { panic!("label and goto statements are not supported!"); }
        }
    }

    fn codegen_ret_stat(&mut self, exps: &Vec<Exp>, last_line: Line) -> Result<()> {
        if exps.is_empty() {
            self.emit_return(last_line, 0, 0);
            Ok(())
        } else {
            if exps.len() == 1 {
                match &exps[0] {
                    Exp::Name(name, line) => {
                        if let Ok(reg) = self.local_var_slot(name) {
                            self.emit_return(*line, reg as isize, 1);
                            return Ok(());
                        }
                    }

//                    Exp::FnCall(fn_call) => {
//                        let reg = self.alloc_register()? as isize;
//                        self.codegen_tail_call_exp(fn_call, reg)?;
//                        self.free_register();
//                        self.emit_return(last_line, reg, -1);
//                    }

                    _ => {}
                }
            }

            let mult_ret = is_vararg_or_fn_call(exps.last().unwrap());
            let num = exps.len() - 1;
            for (i, exp) in exps.iter().enumerate() {
                let reg = self.alloc_register()? as isize;
                // has `...` or function call
                if i == num && mult_ret {
                    self.codegen_exp(exp, reg, -1)?;
                } else {
                    self.codegen_exp(exp, reg, 1)?;
                }
            }
            self.free_registers(num);

            let a = self.used_regs;
            if mult_ret {
                self.emit_return(last_line, a as isize, -1);
            } else {
                self.emit_return(last_line, a as isize, exps.len() as isize);
            }

            Ok(())
        }
    }

    // local function f() end => local f; f = function() end
    fn codegen_local_fn_def_stat(&mut self, name: &String, fn_def: &FnDef) -> Result<()> {
        let reg = self.add_local_var(name.clone(), self.pc() + 2)?;
        self.codegen_fn_def_exp(fn_def, reg as isize)
    }

    fn codegen_fn_call_stat(&mut self, fn_call: &FnCall) -> Result<()> {
        let reg = self.alloc_register()?;
        self.codegen_fn_call_exp(fn_call, reg as isize, 0)?;
        self.free_register();
        Ok(())
    }

    fn codegen_break_stat(&mut self, line: Line) -> Result<()> {
        let pc = self.emit_jmp(line, 0, 0);
        self.add_break_jump(pc, line)
    }

    fn codegen_do_stat(&mut self, block: &Block) -> Result<()> {
        // not a loop block
        self.enter_scope(false);
        self.codegen_block(block)?;
        self.close_open_up_values(block.last_line);
        self.exit_scope()
    }

    /*
            ______________
           |  false? jmp  |
           V              /
    repeat block until exp
    */
    fn codegen_repeat_stat(&mut self, exp: &Exp, block: &Block) -> Result<()> {
        self.enter_scope(true);

        let pc_before_block = self.pc();
        self.codegen_block(block)?;

        let reg = self.alloc_register()?;
        self.codegen_exp(exp, reg as isize, 1)?;
        self.free_register();

        self.emit_test(block.last_line, reg as isize, 0);
        let a = self.get_jump_arg_a();
        self.emit_jmp(block.last_line, a, (pc_before_block as isize - self.pc() as isize - 1));
        self.close_open_up_values(block.last_line);

        self.exit_scope()
    }

    /*
               ______________
              /  false? jmp  |
             /               |
    while exp do block end <-'
          ^           \
          |___________/
               jmp
    */
    fn codegen_while_stat(&mut self, exp: &Exp, block: &Block) -> Result<()> {
        let pc_before_exp = self.pc();

        let reg = self.alloc_register()?;
        self.codegen_exp(exp, reg as isize, 1)?;
        self.free_register();

        self.emit_test(block.last_line, reg as isize, 0);
        let pc_jmp_to_end = self.emit_jmp(block.last_line, 0, 0);

        self.enter_scope(true);
        self.codegen_block(block)?;
        self.close_open_up_values(block.last_line);
        self.emit_jmp(block.last_line, 0, (pc_before_exp as isize - self.pc() as isize - 1));
        self.exit_scope()?;

        self.fix_sbx(pc_jmp_to_end, (self.pc() - pc_jmp_to_end) as isize);

        Ok(())
    }

    /*
             _________________       _________________       _____________
            / false? jmp      |     / false? jmp      |     / false? jmp  |
           /                  V    /                  V    /              V
    if exp1 then block1 elseif exp2 then block2 elseif true then block3 end <-.
                       \                       \                       \      |
                        \_______________________\_______________________\_____|
                        jmp                     jmp                     jmp
    */
    fn codegen_condition_stat(&mut self, exps: &Vec<Exp>, blocks: &Vec<Block>) -> Result<()> {
        let mut pc_jmp_to_ends = vec![];
        let mut pc_jmp_to_next_exp: isize = -1;

        for (i, exp) in exps.iter().enumerate() {
            if pc_jmp_to_next_exp >= 0 {
                self.fix_sbx(pc_jmp_to_next_exp as usize, self.pc() as isize - pc_jmp_to_next_exp as isize);
            }

            let reg = self.alloc_register()? as isize;
            self.codegen_exp(exp, reg, 1)?;
            self.free_register();

            self.emit_test(0, reg, 0);
            pc_jmp_to_next_exp = self.emit_jmp(0, 0, 0) as isize;

            self.enter_scope(false);
            let block = &blocks[i];
            self.codegen_block(block)?;
            self.close_open_up_values(block.last_line);
            self.exit_scope()?;

            if i < exps.len() - 1 {
                pc_jmp_to_ends.push(self.emit_jmp(block.last_line, 0, 0));
            } else {
                pc_jmp_to_ends.push(pc_jmp_to_next_exp as usize);
            }
        }

        for pc in pc_jmp_to_ends {
            self.fix_sbx(pc, self.pc() as isize - pc as isize);
        }

        Ok(())
    }

    fn codegen_for_num_stat(&mut self, for_num: &ForNum) -> Result<()> {
        // need OP_FORPREP and OP_FORLOOP
        self.enter_scope(true);

        let names = vec![
            "(for index)".to_string(),
            "(for limit)".to_string(),
            "(for step)".to_string(),
        ];

        self.codegen_local_var_decl_stat_borrow(&names, &vec![&for_num.init, &for_num.limit, &for_num.step], for_num.line_of_do)?;
        self.add_local_var(for_num.name.clone(), self.pc() + 2)?;

        let a = self.used_regs as isize - 4;
        let pc_for_prep = self.emit_for_prep(for_num.line_of_do, a, 0);
        self.codegen_block(&*for_num.block)?;
        self.close_open_up_values(for_num.block.last_line);
        let pc_for_loop = self.emit_for_loop(for_num.line_of_for, a, 0);


        self.fix_sbx(pc_for_prep as usize, pc_for_loop - pc_for_prep - 1);
        self.fix_sbx(pc_for_loop as usize, pc_for_prep - pc_for_loop);

        self.exit_scope()
    }

    fn codegen_for_in_stat(&mut self, for_in: &ForIn, line: Line) -> Result<()> {
        self.enter_scope(true);

        let names = vec![
            "(for generator)".to_string(),
            "(for state)".to_string(),
            "(for control)".to_string(),
        ];
        self.codegen_local_var_decl_stat(&names, &for_in.exp_list, line)?;

        for name in for_in.name_list.iter() {
            self.add_local_var(name.clone(), self.pc() + 2)?;
        }

        let pc_jmp_to_tfc = self.emit_jmp(line, 0, 0);
        self.codegen_block(&*for_in.block)?;
        self.close_open_up_values(for_in.block.last_line);
        self.fix_sbx(pc_jmp_to_tfc, self.pc() as isize - pc_jmp_to_tfc as isize);

        let reg_gen = self.local_var_slot(&"(for generator)".to_string())?;
        self.emit_t_for_call(line, reg_gen as isize, for_in.name_list.len() as isize);
        self.emit_t_for_loop(line, reg_gen as isize + 2, pc_jmp_to_tfc as isize - self.pc() as isize - 1);

        self.exit_scope()
    }


    fn codegen_assign_stat(&mut self, names: &Vec<Exp>, vals: &Vec<Exp>, line: Line) -> Result<()> {
        let old_regs = self.used_regs;
        let mut t_regs = iter::repeat(isize::default()).take(names.len()).collect::<Vec<_>>();
        let mut k_regs = iter::repeat(isize::default()).take(names.len()).collect::<Vec<_>>();
        let mut v_regs = iter::repeat(isize::default()).take(names.len()).collect::<Vec<_>>();

        for (i, name_exp) in names.iter().enumerate() {
            if let Exp::TableAccess(prefix_exp, key_exp, line) = name_exp {
                t_regs[i] = self.alloc_register()? as isize;
                self.codegen_exp(&*prefix_exp, t_regs[i], 1)?;
                k_regs[i] = self.alloc_register()? as isize;
                self.codegen_exp(&*key_exp, k_regs[i], 1)?;
            } else {
                match name_exp {
                    Exp::Name(name, line) => {
                        if self.local_var_slot(name).is_err() && self.up_value_index(name).is_none() {
                            // global variable
                            k_regs[i] = -1;
                            if self.constant_index(&Constant::String(name.clone())) > 0xFF {
                                k_regs[i] = self.alloc_register()? as isize;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        for i in 0..names.len() {
            v_regs[i] = (self.used_regs + i) as isize;
        }

        if vals.len() >= names.len() {
            for (i, exp) in vals.iter().enumerate() {
                let a = self.alloc_register()? as isize;
                if i >= names.len() && i == vals.len() - 1 && is_vararg_or_fn_call(exp) {
                    self.codegen_exp(exp, a, 0)?;
                } else {
                    self.codegen_exp(exp, a, 1)?;
                }
            }
        } else {
            let mut mult_ret = false;
            for (i, exp) in vals.iter().enumerate() {
                let a = self.alloc_register()? as isize;
                if i == vals.len() - 1 && is_vararg_or_fn_call(exp) {
                    mult_ret = true;

                    let n = names.len() - vals.len() + 1;
                    self.codegen_exp(exp, a, n as isize)?;
                    self.alloc_registers(n - 1)?;
                } else {
                    self.codegen_exp(exp, a, 1)?;
                }
            }

            if !mult_ret {
                let n = names.len() - vals.len();
                let a = self.alloc_registers(n)? as isize;
                self.emit_load_nil(line, a, n as isize);
            }
        }
        let last_line = line;
        for (i, exp) in names.iter().enumerate() {
            match exp {
                Exp::Name(name, line) => {
                    if let Ok(a) = self.local_var_slot(name) {
                        self.emit_move(last_line, a as isize, v_regs[i]);
                    } else if let Some(b) = self.up_value_index(name) {
                        self.emit_set_up_value(last_line, v_regs[i], b as isize);
                    } else if let Ok(a) = self.local_var_slot(&"_ENV".to_string()) {
                        if k_regs[i] < 0 {
                            let b = 0x100 + self.constant_index(&Constant::String(name.clone())) as isize;
                            self.emit_set_table(last_line, a as isize, b, v_regs[i]);
                        } else {
                            self.emit_set_table(last_line, a as isize, k_regs[i], v_regs[i]);
                        }
                    } else {
                        let a = self.up_value_index(&"_ENV".to_string())
                            .ok_or(Error::NotUpValue {
                                line: last_line,
                            })? as isize;

                        if k_regs[i] < 0 {
                            let b = 0x100 + self.constant_index(&Constant::String(name.clone())) as isize;
                            self.emit_set_table_up(last_line, a, b, v_regs[i]);
                        } else {
                            self.emit_set_table_up(last_line, a, k_regs[i], v_regs[i]);
                        }
                    }
                }

                _ => {
                    self.emit_set_table(last_line, t_regs[i], k_regs[i], v_regs[i]);
                }
            }
        }

        self.used_regs = old_regs;
        Ok(())
    }

    fn codegen_local_var_decl_stat(&mut self, names: &Vec<String>, exps: &Vec<Exp>, last_line: Line) -> Result<()> {
        let old_regs = self.used_regs;
        if exps.len() == names.len() {
            for exp in exps.iter() {
                let a = self.alloc_register()?;
                self.codegen_exp(exp, a as isize, 1)?;
            }
        } else if exps.len() > names.len() {
            for (i, exp) in exps.iter().enumerate() {
                let a = self.alloc_register()?;
                if i == exps.len() - 1 && is_vararg_or_fn_call(exp) {
                    self.codegen_exp(exp, a as isize, 0)?;
                } else {
                    self.codegen_exp(exp, a as isize, 1)?;
                }
            }
        } else {
            let mut mult_ret = false;
            for (i, exp) in exps.iter().enumerate() {
                let a = self.alloc_register()?;
                if i == exps.len() - 1 && is_vararg_or_fn_call(exp) {
                    mult_ret = true;
                    let n = names.len() - exps.len() + 1;
                    self.codegen_exp(exp, a as isize, n as isize)?;
                    self.alloc_registers(n - 1)?;
                } else {
                    self.codegen_exp(exp, a as isize, 1)?;
                }
            }

            if !mult_ret {
                let n = names.len() - exps.len();
                let a = self.alloc_registers(n)?;
                self.emit_load_nil(last_line, a as isize, n as isize);
            }
        }

        self.used_regs = old_regs;
        let start_pc = self.pc() + 1;
        for name in names {
            self.add_local_var(name.clone(), start_pc)?;
        }

        Ok(())
    }

    fn codegen_local_var_decl_stat_borrow(&mut self, names: &Vec<String>, exps: &Vec<&Exp>, last_line: Line) -> Result<()> {
        let old_regs = self.used_regs;
        if exps.len() == names.len() {
            for exp in exps.iter() {
                let a = self.alloc_register()?;
                self.codegen_exp(exp, a as isize, 1)?;
            }
        } else if exps.len() > names.len() {
            for (i, exp) in exps.iter().enumerate() {
                let a = self.alloc_register()?;
                if i == exps.len() - 1 && is_vararg_or_fn_call(exp) {
                    self.codegen_exp(exp, a as isize, 0)?;
                } else {
                    self.codegen_exp(exp, a as isize, 1)?;
                }
            }
        } else {
            let mut mult_ret = false;
            for (i, exp) in exps.iter().enumerate() {
                let a = self.alloc_register()?;
                if i == exps.len() - 1 && is_vararg_or_fn_call(exp) {
                    mult_ret = true;
                    let n = names.len() - exps.len() + 1;
                    self.codegen_exp(exp, a as isize, n as isize)?;
                    self.alloc_registers(n - 1)?;
                } else {
                    self.codegen_exp(exp, a as isize, 1)?;
                }
            }

            if !mult_ret {
                let n = names.len() - exps.len();
                let a = self.alloc_registers(n)?;
                self.emit_load_nil(last_line, a as isize, n as isize);
            }
        }
        self.used_regs = old_regs;
        let start_pc = self.pc() + 1;
        for name in names {
            self.add_local_var(name.clone(), start_pc)?;
        }

        Ok(())
    }
}

/********************** expression code generation ***********************/

impl FnInfo {
    pub fn codegen_exp(&mut self, exp: &Exp, a: isize, n: isize) -> Result<()> {
        match exp {
            Exp::Nil(line) => {
                self.emit_load_nil(*line, a, n);
                Ok(())
            }
            Exp::False(line) => {
                self.emit_load_bool(*line, a, 0, 0);
                Ok(())
            }
            Exp::True(line) => {
                self.emit_load_bool(*line, a, 1, 0);
                Ok(())
            }
            Exp::Integer(num, line) => {
                self.emit_load_k(*line, a, Constant::Integer(*num));
                Ok(())
            }
            Exp::Float(num, line) => {
                self.emit_load_k(*line, a, Constant::Number(*num));
                Ok(())
            }
            Exp::String(s, line) => {
                self.emit_load_k(*line, a, Constant::String(s.clone()));
                Ok(())
            }
            Exp::Name(name, line) => self.codegen_name_exp(name, a, n, *line),
            Exp::Parens(exp) => self.codegen_exp(&*exp, a, n),
            Exp::Vararg(line) => self.codegen_vararg_exp(a, n, *line),
            Exp::Unop(op, exp, line) => self.codegen_unop_exp(op, exp, a, *line),
            Exp::Binop(exp1, op, exp2, line) => self.codegen_binop_exp(&*exp1, op, &*exp2, a, n, *line),
            Exp::Concat(exps, line) => self.codegen_concat_exp(exps, a, n, *line),
            Exp::TableConstructor(fields, line) => self.codegen_table_constructor_exp(fields, a, *line),
            Exp::TableAccess(obj, key, line) => self.codegen_table_access_exp(&*obj, &*key, a, n, *line),
            Exp::FnDef(fn_def) => self.codegen_fn_def_exp(fn_def, a),
            Exp::FnCall(fn_call) => self.codegen_fn_call_exp(fn_call, a, n),
        }
    }

    fn codegen_name_exp(&mut self, name: &String, a: isize, n: isize, line: Line) -> Result<()> {

        let reg = self.local_var_slot(name);
        if let Ok(reg) = reg {
            self.emit_move(line, a, reg as isize);
            Ok(())
        } else if let Some(idx) = self.up_value_index(name) {
            self.emit_get_up_value(line, a, idx as isize);
            Ok(())
        } else {
            // x => _Env['x']
            self.codegen_table_access_exp(&Exp::Name("_ENV".to_string(), 0), &Exp::String(name.clone(), 0), a, 0, line)
        }
    }

    // f[a] := function(args) body end
    fn codegen_fn_def_exp(&mut self, fn_def: &FnDef, a: isize) -> Result<()> {
        // fixme
        let parent = FnInfoRef(Rc::new(RefCell::new(self.clone())));
        let sub_fn = Self::new(Some(parent), fn_def.par_list.clone(), fn_def.line, fn_def.last_line);
        let mut sub_fn = FnInfoRef(Rc::new(RefCell::new(sub_fn)));
        self.sub_fns.push(sub_fn.clone());

        for param in &fn_def.par_list.params {
            sub_fn.0.borrow_mut().add_local_var(param.clone(), 0)?;
        }
        sub_fn.0.borrow_mut().codegen_block(&*fn_def.block)?;
//        self.codegen_block(&*fn_def.block)?;
        sub_fn.0.borrow_mut().exit_scope()?;
        sub_fn.0.borrow_mut().emit_return(fn_def.block.last_line, 0, 0);

        let bx = self.sub_fns.len() - 1;
        self.emit_closure(fn_def.block.last_line, a, bx as isize);
        Ok(())
    }

    // f[a] := function(args) body end
    fn codegen_fn_def_exp2(parent: &FnInfoRef, fn_def: &FnDef, a: isize) -> Result<()> {
        // fixme
        let sub_fn = Self::new(Some(parent.clone()), fn_def.par_list.clone(), fn_def.line, fn_def.last_line);
        let mut sub_fn = FnInfoRef(Rc::new(RefCell::new(sub_fn)));
        parent.0.borrow_mut().sub_fns.push(sub_fn.clone());

        for param in &fn_def.par_list.params {
            sub_fn.0.borrow_mut().add_local_var(param.clone(), 0)?;
        }
        parent.0.borrow_mut().codegen_block(&*fn_def.block)?;
        sub_fn.0.borrow_mut().exit_scope()?;
        sub_fn.0.borrow_mut().emit_return(fn_def.block.last_line, 0, 0);

        let bx = parent.0.borrow_mut().sub_fns.len() - 1;
        parent.0.borrow_mut().emit_closure(fn_def.block.last_line, a, bx as isize);
        Ok(())
    }

    fn codegen_vararg_exp(&mut self, a: isize, n: isize, line: Line) -> Result<()> {
        if !self.is_vararg {
            Err(Error::NotVararg { line })
        } else {
            self.emit_vararg(line, a, n);
            Ok(())
        }
    }

    fn codegen_unop_exp(&mut self, op: &Token, exp: &Exp, a: isize, line: Line) -> Result<()> {
        let b = self.alloc_register()? as isize;
        self.codegen_exp(exp, b, 1)?;
        self.emit_unary_op(line, op, a, b);
        self.free_register();
        Ok(())
    }

    fn codegen_binop_exp(&mut self, exp1: &Exp, op: &Token, exp2: &Exp, a: isize, n: isize, line: Line) -> Result<()> {
        match op {
            Token::OpAnd | Token::OpOr => {
                let b = self.alloc_register()? as isize;
                self.codegen_exp(exp1, b, 1)?;
                self.free_register();
                if *op == Token::OpAnd {
                    self.emit_test_set(line, a, b, 0);
                } else {
                    self.emit_test_set(line, a, b, 1);
                }
                let jmp_pc = self.emit_jmp(line, 0, 0);

                let b = self.alloc_register()? as isize;
                self.codegen_exp(exp2, b, 1)?;
                self.free_register();
                self.emit_move(line, a, b);
                self.fix_sbx(jmp_pc, self.pc() as isize - jmp_pc as isize);
            }

            _ => {
                let b = self.alloc_register()? as isize;
                self.codegen_exp(exp1, b, 1)?;
                let c = self.alloc_register()? as isize;
                self.codegen_exp(exp2, c, 1)?;
                self.emit_binary_op(line, op, a, b, c);
                self.free_registers(2);
            }
        }
        Ok(())
    }

    fn codegen_concat_exp(&mut self, exps: &Vec<Exp>, a: isize, n: isize, line: Line) -> Result<()> {
        for exp in exps {
            let a = self.alloc_register()? as isize;
            self.codegen_exp(exp, a, 1)?;
        }

        let c = self.used_regs - 1;
        let b = c - exps.len() + 1;
        self.free_registers(c + 1 - b);
        self.emit_ABC(line, opcode::OP_CONCAT, a, b as isize, c as isize);
        Ok(())
    }

    fn codegen_table_constructor_exp(&mut self, fields: &Vec<Field>, a: isize, line: Line) -> Result<()> {
        // 有多少个是没有下标的
        let mut n_arr = 0;
        for field in fields {
            if field.key.is_none() {
                n_arr += 1;
            }
        }

        let mult_ret = !fields.is_empty() && is_vararg_or_fn_call(&fields.last().unwrap().val);

        self.emit_new_table(line, a, n_arr, fields.len() as isize - n_arr);
        let mut arr_idx = 0;

        for (i, field) in fields.iter().enumerate() {
            // 先处理数组
            match field.key {
                Some(ref key) => {
                    let b = self.alloc_register()? as isize;
                    self.codegen_exp(key, b, 1)?;
                    let c = self.alloc_register()? as isize;
                    self.codegen_exp(&field.val, c, 1)?;

                    self.free_registers(2);

                    let line = 0;
                    self.emit_set_table(line, a, b, c);
                }

                None => {
                    arr_idx += 1;
                    let tmp = self.alloc_register()? as isize;
                    if i == fields.len() - 1 && mult_ret {
                        self.codegen_exp(&field.val, tmp, -1)?;
                    } else {
                        self.codegen_exp(&field.val, tmp, 1)?;
                    }

                    // LFIELDS_PER_FLUSH
                    if arr_idx % 50 == 0 || arr_idx == fields.len() {
                        let n = if arr_idx % 50 == 0 {
                            50
                        } else {
                            arr_idx % 50
                        };

                        let c = (arr_idx - 1) / 50 + 1;
                        self.free_registers(n);
                    }
                }
            }
        }
        Ok(())
    }

    fn codegen_table_access_exp(&mut self, obj: &Exp, key: &Exp, a: isize, n: isize, line: Line) -> Result<()> {
        let old_Regs = self.used_regs;
        let b = self.alloc_register()? as isize;
        self.codegen_exp(obj, b, 1)?;
        let c = self.alloc_register()? as isize;
        self.codegen_exp(key, c, 1)?;
        self.emit_get_table(line, a, b, c);
        self.free_registers(2);
        Ok(())
    }

    fn codegen_fn_call_exp(&mut self, fn_call: &FnCall, a: isize, n: isize) -> Result<()> {
        let n_args = self.prep_fn_call(fn_call, a)? as isize;
        self.emit_call(fn_call.line, a, n_args, n);
        Ok(())
    }

    fn codegen_tail_call_exp(&mut self, fn_call: &FnCall, a: isize) -> Result<()> {
        unimplemented!();
    }

    fn prep_fn_call(&mut self, fn_call: &FnCall, a: isize) -> Result<usize> {
        let mut n_args = fn_call.args.len();
        let mut last_arg_is_vararg_or_fn_call = false;
        self.codegen_exp(&*fn_call.prefix, a, 1)?;

        // fixme
        match fn_call.name {
            Some(ref name) => {
                match **name {
                    Exp::String(ref s, line) => {
                        let constant = Constant::String(s.clone());
                        let c = 0x100 + self.constant_index(&constant) as isize;
                        self.emit_self(line, a, a, c)
                    }
                    _ => unreachable!()
                };
            }
            _ => {}
        };

        for (i, arg) in fn_call.args.iter().enumerate() {
            let tmp = self.alloc_register()? as isize;
            if i == n_args - 1 && is_vararg_or_fn_call(arg) {
                last_arg_is_vararg_or_fn_call = true;
                self.codegen_exp(arg, tmp, -1)?;
            } else {
                self.codegen_exp(arg, tmp, 1)?;
            }
        }

        self.free_registers(n_args);

        if let Some(_) = fn_call.name {
            n_args += 1;
        }

        if last_arg_is_vararg_or_fn_call {
            n_args -= 1;
        }

        Ok(n_args)
    }
}

#[inline]
fn is_vararg_or_fn_call(exp: &Exp) -> bool {
    match exp {
        Exp::Vararg(_) => true,
        Exp::FnCall(_) => true,
        _ => false,
    }
}


mod tests {
    use std::fs;

    use crate::binary::encode;
    use crate::compiler::lexer::*;
    use crate::compiler::parser::*;

    use super::*;

    #[test]
    fn test_codegen() {
        let s = r##"
        local g = {
            a = 1,
            b = {}
        }
        -- comment
        local a = true and false or false or not true
        local b = ((1 | 2) & 3) >> 1 << 1
        local c = (3 + 2 - 1) * (5 % 2) // 2 / 2 ^ 2
        local d = not not not not not false
        local e = - - - - -1
        local f = ~ ~ ~ ~ ~1

        package.preload.mymod = function(modname)
          local loader = function(modname, extra)
            print("loading")
          end
          return loader, ""
        end

        co = function () print("hello") end

        function hello()
          function world()
             a = 1
             while a < 10 do
                a = a + 1
             end
           end
        end



        "##.to_string();

//        let s  = r##"
//        function preloadSearcher(modname)
//          if package.preload[modname] ~= nil then
//            return package.preload[modname]
//          else
//            return 1
//          end
//        end5
//        local a = 1;
//        "##.to_string();
        let mut lexer = Lexer::from_iter(s.into_bytes(), "test".to_string());
        let block = parse_block(&mut lexer).expect("parse error");
        println!("{:#?}", block);
        let proto = gen_prototype(Box::new(block));
        println!("{:#?}", proto);
    }

    #[test]
    fn test_codegen2() {
        let s = r##"
        local g = {
            a = 1,
            b = {}
        }

        print(g.a)
        print(g.b)

        local a = 1
        while a < 1 do
            a = a + 1
        end
        print(a)
        "##.to_string();

        let mut lexer = Lexer::from_iter(s.into_bytes(), "test".to_string());
        let block = parse_block(&mut lexer).expect("parse error");
        let proto = gen_prototype(Box::new(block));

        let bytes = encode(proto.unwrap(), Some("@hello2.lua".to_string()));
        fs::write("./tests/test.out", bytes);
    }
}