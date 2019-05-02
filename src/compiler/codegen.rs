#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use core::borrow::BorrowMut;
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::ast::{Block, Exp, Stat};
use crate::compiler::error::{Error, Result};

const MAXARG_BX: isize = (1 << 18) - 1;
// 262143
const MAXARG_SBX: isize = MAXARG_BX >> 1; // 131071

#[derive(Debug)]
struct LocalVarInfo {
    scope_level: usize,
    slot: usize,
    is_captured: bool,
}

#[derive(Debug, Copy, Clone)]
struct UpValueInfo {
    local_var_slot: usize,
    up_value_index: usize,
    /// The sequent of UpValue in Foreign Function
    index: usize,
}

/// 符号表的设计：作用域链
#[derive(Debug)]
struct FnInfo {
    constants: HashMap<String, usize>,
    /// num of used regs
    used_regs: usize,
    /// maximum need of num of regs
    max_regs: usize,
    /// Block scope level
    scope_level: usize,
    /// Local variable of all scope
    local_vars: Vec<HashMap<String, Option<Rc<LocalVarInfo>>>>,
    /// Record some breaks statements
    breaks: Vec<Option<Vec<usize>>>,
    /// Parents' index
    parent: Option<Rc<FnInfo>>,
    /// UpValues
    up_values: HashMap<String, UpValueInfo>,
    /// Store Lua instructions
    instructions: Vec<u32>,
    /// nested Functions
    sub_fns: Vec<FnInfo>,
    /// The function's param num
    num_params: usize,
    /// has `...`
    is_var_arg: bool,
}


impl FnInfo {
    /// Create a FnInfo structure
    #[inline]
    fn new(parent: Option<Rc<FnInfo>>, fn_def: Exp) -> Self {
        let (num_params, is_var_arg) = match fn_def {
            Exp::FnDef(par_list, _, _, _) => {
                (par_list.params.len(), par_list.is_vararg)
            }
            _ => unreachable!()
        };

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
            is_var_arg,
        }
    }

    fn index_of_constant(&mut self, k: String) -> usize {
        unimplemented!()
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

        // 324
        unimplemented!();

        self.scope_level -= 1;
        match self.local_vars.pop() {
            Some(vars) => Ok(()),
            None => Err(Error::NoMoreScopes)
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
            Some(upvalue) => {
                return Ok(upvalue.index);
            }
            _ => {}
        }

        // 325
        unimplemented!()
    }


    #[inline]
    fn emit_ABC(&mut self, opcode: u32, a: u32, b: u32, c: u32) {
        let ins = b << 23 | c << 14 | a << 6 | opcode;
        self.instructions.push(ins);
    }


    #[inline]
    fn emit_ABx(&mut self, opcode: u32, a: u32, bx: u32) {
        let ins = bx << 14 | a << 6 | opcode;
        self.instructions.push(ins);
    }

    #[inline]
    fn emit_AsBx(&mut self, opcode: u32, a: u32, b: u32) {
        let ins = (b + MAXARG_SBX as u32) << 14 | a << 6 | opcode;
        self.instructions.push(ins);
    }

    #[inline]
    fn emit_Ax(&mut self, opcode: u32, ax: u32) {
        let ins = ax << 6 | opcode;
        self.instructions.push(ins);
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

fn codegen_block(fn_info: &mut FnInfo, node: &Block) {
    for stat in &node.stats {
        codegen_stat(fn_info, stat);
    }

    codegen_ret_stats(fn_info, &node.ret_exps);
}

fn codegen_ret_stats(fn_info: &mut FnInfo, exps: &Vec<Exp>) {
    unimplemented!()
}

fn codegen_stat(fn_info: &mut FnInfo, stat: &Stat) {
    match stat {
        Stat::FnCall(fn_call, line, last_line) => { unimplemented!() }
        _ => {}
    }
}

/****************** statement code generation *********************/

fn codegen_local_fn_def_stat(fn_info: &mut FnInfo, name: String, exp: &Exp) -> Result<()> {
    let reg = fn_info.borrow_mut().add_local_var(name)?;
    codegen_fn_def_exp(fn_info, exp)?;
    Ok(())
}


/****************** expression code generation *********************/

fn codegen_fn_def_exp(fn_info: &mut FnInfo, stat: &Exp) -> Result<()> {
    unimplemented!()
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