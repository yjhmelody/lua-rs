#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_snake_case)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::binary::chunk::{Constant, Prototype};
use crate::compiler::ast::{Block, Exp, Field, FnCall, ForIn, ForNum, ParList, Stat};
use crate::compiler::error::{Error, Result};
use crate::compiler::lexer::Line;
use crate::compiler::token::Token;
use crate::vm::opcode;

/// 262143
const MAXARG_BX: isize = (1 << 18) - 1;
/// 131071
const MAXARG_SBX: isize = MAXARG_BX >> 1;

/// Local Variable Information
#[derive(Debug)]
struct LocalVarInfo {
    scope_level: usize,
    slot: usize,
    is_captured: bool,
}

/// Up Value Information
#[derive(Debug, Copy, Clone)]
struct UpValueInfo {
    local_var_slot: usize,
    up_value_index: usize,
    /// The sequent of UpValue in Foreign Function
    index: usize,
}

/// Function Information Table for Lua
#[derive(Debug)]
struct FnInfo {
    constants: HashMap<Constant, usize>,
    /// Num of used regs
    used_regs: usize,
    /// Maximum need of num of regs
    max_regs: usize,
    /// Block scope level
    scope_level: usize,
    /// Local variable of all scope
    local_vars: Vec<HashMap<String, Option<Rc<LocalVarInfo>>>>,
    /// Record some breaks statements
    breaks: Vec<Option<Vec<usize>>>,
    /// Parents' index
    parent: Option<Rc<RefCell<FnInfo>>>,
    /// UpValues
    up_values: HashMap<String, UpValueInfo>,
    /// Store Lua instructions
    instructions: Vec<u32>,
    /// Nested Functions
    sub_fns: Vec<FnInfo>,
    /// The function's param num
    num_params: usize,
    /// Has `...`
    is_vararg: bool,
}

/********************** keep function information ************************/

impl FnInfo {
    /// Create a FnInfo structure
    #[inline]
    fn new(parent: Option<Rc<RefCell<FnInfo>>>, par_list: ParList, block: Box<Block>) -> Self {
        let is_vararg = par_list.is_vararg;
        let num_params = par_list.params.len();
        Self {
            constants: HashMap::new(),
            used_regs: 0,
            max_regs: 0,
            scope_level: 0,
            local_vars: Vec::new(),
            breaks: Vec::new(),
            parent,
            up_values: HashMap::new(),
            instructions: Vec::new(),
            sub_fns: Vec::new(),
            num_params,
            is_vararg,
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
    fn get_current_scope(&self) -> &HashMap<String, Option<Rc<LocalVarInfo>>> {
        &self.local_vars[self.scope_level]
    }

    #[inline]
    fn get_current_scope_mut(&mut self) -> &mut HashMap<String, Option<Rc<LocalVarInfo>>> {
        &mut self.local_vars[self.scope_level]
    }

    /// Create a new scope for vars
    #[inline]
    fn enter_scope(&mut self, breakable: bool) {
        self.scope_level += 1;
        if breakable {
            self.breaks.push(Some(vec![]));
        } else {
            self.breaks.push(None);
        }
    }

    /// Exit current scope
    #[inline]
    fn exit_scope(&mut self) -> Result<()> {
        let jump = self.breaks.pop().ok_or(Error::NoMoreScopes)?;
        unimplemented!();
        self.scope_level -= 1;
        match self.local_vars.pop() {
            Some(vars) => Ok(()),
            None => Err(Error::NoMoreScopes)
        }
    }

    fn get_jump_arg_a(&mut self) -> u32 {
        let mut has_captured_local_var = false;
        let mut min_local_var_slot = self.max_regs;
        let local_vars = self.get_current_scope_mut();
        local_vars.clone().iter().for_each(|(k, v)| {
            match v {
                Some(local_var) => {
                    if local_var.is_captured {
                        has_captured_local_var = true;
                    }
                    // todo: fix it
                    if local_var.slot < min_local_var_slot {
                        min_local_var_slot = local_var.slot;
                    }
                }
                _ => {}
            }
        });

        if has_captured_local_var {
            min_local_var_slot as u32 + 1
        } else {
            0
        }
    }

    /// Add a local variable and return register index
    fn add_local_var(&mut self, name: String) -> Result<usize> {
        let new_var = Rc::new(LocalVarInfo {
            scope_level: self.scope_level,
            slot: self.alloc_register()?,
            is_captured: false,
        });

        let slot = new_var.slot;
        self.local_vars[self.scope_level].insert(name, Some(new_var));
        Ok(slot)
    }

    /// Get name's register number
    fn local_var_slot(&self, name: &String) -> Result<usize> {
        match self.get_current_scope().get(name) {
            Some(Some(local_var)) => Ok(local_var.slot),
            Some(None) => Ok(0),
            _ => Err(Error::IllegalRegister),
        }
    }

    /// Remove a variable from current scope
    #[inline]
    fn remove_local_var(&mut self, name: String) {
        self.free_register();
        self.get_current_scope_mut().insert(name, None);
    }

    /// Create a jump instruction to a latest loop block
    fn add_break_jump(&mut self, pc: usize) -> Result<()> {
        for brk in &mut self.breaks.iter_mut().rev() {
            match brk.as_mut() {
                Some(arr) => {
                    arr.push(pc);
                    return Ok(());
                }

                None => {}
            }
        }
        Err(Error::NoLoop)
    }

    /// Get up value's index
    fn up_value_index(&mut self, name: &String) -> Result<usize> {
        match self.up_values.get(name) {
            Some(up_value) => {
                return Ok(up_value.index);
            }
            _ => {}
        }

        // 325
        unimplemented!()
    }

    fn close_open_up_values(&mut self, line: Line) {
        let a = self.get_jump_arg_a();
        if a > 0 {
            self.emit_jump(line, a, 0);
        }
    }

    fn to_prototype(&self) -> Prototype {
        let constants = self.constants.iter();
        unimplemented!()
    }
}

/********************** emit bytecode ************************/

impl FnInfo {
    #[inline]
    fn emit_ABC(&mut self, line: Line, opcode: u8, a: u32, b: u32, c: u32) {
        let ins = b << 23 | c << 14 | a << 6 | opcode as u32;
        self.instructions.push(ins);
    }

    #[inline]
    fn emit_ABx(&mut self, line: Line, opcode: u8, a: u32, bx: u32) {
        let ins = bx << 14 | a << 6 | opcode as u32;
        self.instructions.push(ins);
    }

    #[inline]
    fn emit_AsBx(&mut self, line: Line, opcode: u8, a: u32, b: u32) {
        let ins = (b + MAXARG_SBX as u32) << 14 | a << 6 | opcode as u32;
        self.instructions.push(ins);
    }

    #[inline]
    fn emit_Ax(&mut self, line: Line, opcode: u8, ax: u32) {
        let ins = ax << 6 | opcode as u32;
        self.instructions.push(ins);
    }

    /// pc+=sBx; if (a) close all upvalues >= r[a - 1]
    #[inline]
    fn emit_jump(&mut self, line: Line, a: u32, sBx: u32) -> usize {
        self.emit_AsBx(line, opcode::OP_JMP, a, sBx);
        self.instructions.len() - 1
    }

    /// r[a], r[a+1], ..., r[a+b] = nil
    fn emit_load_nil(&mut self, line: Line, a: u32, b: u32) {
        self.emit_ABC(line, opcode::OP_LOADNIL, a, b - 1, 0);
    }

    /// r[a] = b; if (c) pc++
    fn emit_load_bool(&mut self, line: Line, a: u32, b: u32, c: u32) {
        self.emit_ABC(line, opcode::OP_LOADBOOL, a, b, c);
    }

    /// r[a] = kst[bx]
    fn emit_load_k(&mut self, line: Line, a: u32, k: Constant) {
        let idx = self.constant_index(&k) as u32;
        if idx < (1 << 18) {
            self.emit_ABx(line, opcode::OP_LOADK, a, idx);
        } else {
            self.emit_ABx(line, opcode::OP_LOADKX, a, 0);
            self.emit_Ax(line, opcode::OP_EXTRAARG, idx);
        }
    }

    #[inline]
    fn pc(&self) -> usize {
        self.instructions.len() - 1
    }

    fn fix_sbx(&mut self, pc: usize, sBx: isize) {
        let mut ins = self.instructions[pc];
        // clear sBx Op
        ins = ins << 18 >> 18;
        // reset sBx op
        ins = ins | (sBx as u32 + MAXARG_SBX as u32) << 14;
        self.instructions[pc] = ins;
        unimplemented!()
    }
}

/********************** statement code generation ************************/

impl FnInfo {
    fn codegen_block(&mut self, block: &Block) -> Result<()> {
        for stat in &block.stats {
            self.codegen_stat(stat);
        }
        self.codegen_ret_stats(&block.ret_exps)
    }

    fn codegen_stat(&mut self, stat: &Stat) -> Result<()> {
        match stat {
            Stat::FnCall(fn_call, line, last_line) => self.codegen_fn_call_stat(fn_call),
            Stat::Break(line) => self.codegen_break_stat(*line),
            Stat::Do(block) => self.codegen_do_stat(&*block),
            Stat::Repeat(exp, block) => self.codegen_repeat_stat(exp, &*block),
            Stat::While(exp, block) => self.codegen_while_stat(exp, &*block),
            Stat::Condition(exps, blocks) => self.codegen_condition_stat(exps, blocks),
            Stat::ForNum(for_num, line1, line2) => self.codegen_for_num_stat(&*for_num, *line1, *line2),
            Stat::ForIn(for_in, line) => self.codegen_for_in_stat(&*for_in, *line),
            Stat::Assign(names, vals, line) => self.codegen_assign_stat(names, vals, *line),
            Stat::LocalVarDecl(names, exps, line) => self.codegen_local_var_decl_stat(names, exps, *line),
            Stat::LocalFnDef(name, exp) => self.codegen_local_fn_def_stat(name, exp),

            _ => { panic!("label and goto statements are not supported!"); }
        }
    }

    fn codegen_ret_stats(&mut self, exps: &Vec<Exp>) -> Result<()> {
        unimplemented!()
    }

    fn codegen_local_fn_def_stat(&mut self, name: &String, exp: &Exp) -> Result<()> {
        let reg = self.add_local_var(name.clone())?;
        unimplemented!();
        Ok(())
    }

    fn codegen_fn_call_stat(&mut self, fn_call: &FnCall) -> Result<()> {
        let reg = self.alloc_register()?;
        self.codegen_fn_call_exp(fn_call, reg, 0)
    }

    fn codegen_break_stat(&mut self, line: Line) -> Result<()> {
        let pc = self.emit_jump(line, 0, 0);
        self.add_break_jump(pc)
    }

    fn codegen_do_stat(&mut self, block: &Block) -> Result<()> {
        // not a loop block
        self.enter_scope(false);
        self.codegen_block(block)?;
        self.close_open_up_values(block.last_line);
        self.exit_scope()
    }

    fn codegen_repeat_stat(&mut self, exp: &Exp, block: &Block) -> Result<()> {
        unimplemented!()
    }

    fn codegen_while_stat(&mut self, exp: &Exp, block: &Block) -> Result<()> {
        let pc_before_exp = self.pc();
        let reg = self.alloc_register();
        unimplemented!()
    }

    fn codegen_condition_stat(&mut self, exp: &Vec<Exp>, block: &Vec<Block>) -> Result<()> {
        unimplemented!()
    }

    fn codegen_for_num_stat(&mut self, for_num: &ForNum, line1: Line, line2: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_for_in_stat(&mut self, for_in: &ForIn, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_assign_stat(&mut self, names: &Vec<Exp>, vals: &Vec<Exp>, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_local_var_decl_stat(&mut self, names: &Vec<String>, exps: &Vec<Exp>, line: Line) -> Result<()> {
        unimplemented!()
    }
}

/********************** expression code generation ***********************/

impl FnInfo {
    fn codegen_exp(&mut self, exp: &Exp, a: u32, n: u32) {
        match exp {
            Exp::Nil(line) => self.emit_load_nil(*line, a, n),
            Exp::False(line) => self.emit_load_bool(*line, a, 0, 0),
            Exp::True(line) => self.emit_load_bool(*line, a, 1, 0),
            Exp::Integer(num, line) => self.emit_load_k(*line, a, Constant::Integer(*num)),
            Exp::Float(num, line) => self.emit_load_k(*line, a, Constant::Number(*num)),
            Exp::String(s, line) => self.emit_load_k(*line, a, Constant::String(s.clone())),
            Exp::Name(name, line) => { self.codegen_name_exp(name, *line); }
            Exp::Parens(exp) => self.codegen_exp(&*exp, a, n),
            Exp::Vararg(line) => { self.codegen_vararg_exp(*line); }
            Exp::Unop(op, exp, line) => {}
            Exp::Binop(exp1, op, exp2, line) => { self.codegen_binop_exp(&*exp1, op, &*exp2, *line); }
            Exp::Concat(exps, line) => { self.codegen_concat_exp(exps, *line); }
            Exp::TableConstructor(fields, line) => { self.codegen_table_constructor_exp(fields, *line); }
            Exp::TableAccess(obj, key, line) => { self.codegen_table_access_exp(&*obj, &*key, *line); }
            Exp::FnDef(par_list, block, line1, line2) => { self.codegen_fn_def_exp(par_list, &*block, *line1, *line2); }
            Exp::FnCall(fn_call, line1, line2) => { self.codegen_fn_call_exp(fn_call, *line1, *line2); }
        }
    }

    fn codegen_name_exp(&mut self, name: &String, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_fn_def_exp(&mut self, par_list: &ParList, block: &Block, line1: Line, line2: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_vararg_exp(&mut self, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_unop_exp(&mut self, op: &Token, exp: &Exp, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_binop_exp(&mut self, exp1: &Exp, op: &Token, exp2: &Exp, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_concat_exp(&mut self, exps: &Vec<Exp>, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_table_constructor_exp(&mut self, fields: &Vec<Field>, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_table_access_exp(&mut self, obj: &Exp, key: &Exp, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_fn_call_exp(&mut self, fn_call: &FnCall, line1: Line, line2: Line) -> Result<()> {
        unimplemented!()
    }
}

#[inline]
fn is_var_arg_or_fn_call(exp: &Exp) -> bool {
    match exp {
        Exp::Vararg(_) => true,
        Exp::FnCall(_, _, _) => true,
        _ => false,
    }
}


mod tests {
    use super::*;

    #[test]
    fn test_codegen() {}
}