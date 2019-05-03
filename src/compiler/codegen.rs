#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_snake_case)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

use crate::binary::chunk::*;
use crate::compiler::ast::*;
use crate::compiler::error::*;
use crate::compiler::lexer::Line;
use crate::compiler::token::Token;
use crate::vm::opcode;

/// 262143
const MAXARG_BX: isize = (1 << 18) - 1;
/// 131071
const MAXARG_SBX: isize = MAXARG_BX >> 1;


pub fn gen_prototype(block: Box<Block>) -> Rc<Prototype> {
    let last_line = block.last_line;
    let fn_def = FnDef::new(ParList::default(), block, 0, last_line);
    let mut fn_info = FnInfo::new(None, ParList::default(), 0, last_line);
    fn_info.add_local_var("_ENV".to_string());
    fn_info.codegen_fn_def_exp(&fn_def, 0);
    fn_info.to_prototype()
}

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
    local_var_slot: Option<usize>,
    up_value_index: Option<usize>,
    /// The sequent of UpValue in Foreign Function
    index: usize,
}

/// Function Information Table for Lua
#[derive(Debug)]
pub struct FnInfo {
    constants: HashMap<Constant, usize>,
    /// Num of used regs
    used_regs: usize,
    /// Maximum need of num of regs
    max_regs: usize,
    /// Block scope level
    scope_level: usize,
    /// Local variable of all scope
    local_vars: Vec<HashMap<String, Rc<LocalVarInfo>>>,
    /// Record some breaks statements
    breaks: Vec<Option<Vec<usize>>>,
    /// Parents' index
    parent: Option<Rc<RefCell<FnInfo>>>,
    /// UpValues
    up_values: HashMap<String, UpValueInfo>,
    /// Store Lua instructions
    instructions: Vec<u32>,
    /// Nested Functions
    sub_fns: Vec<Rc<FnInfo>>,
    /// The function's param num
    num_params: usize,
    /// Has `...`
    is_vararg: bool,
    /// For debug
    line_nums: Vec<u32>,
    line: Line,
    last_line: Line,
}

/********************** keep function information ************************/

impl FnInfo {
    /// Create a FnInfo structure
    #[inline]
    fn new(parent: Option<Rc<RefCell<FnInfo>>>, par_list: ParList, line: Line, last_line: Line) -> Self {
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
    fn get_current_scope(&self) -> &HashMap<String, Rc<LocalVarInfo>> {
        &self.local_vars[self.scope_level]
    }

    #[inline]
    fn get_current_scope_mut(&mut self) -> &mut HashMap<String, Rc<LocalVarInfo>> {
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

    fn get_jump_arg_a(&mut self) -> isize {
        let mut has_captured_local_var = false;
        let mut min_local_var_slot = self.max_regs;
        let local_vars = self.get_current_scope_mut();
        local_vars.clone().iter().for_each(|(k, local_var)| {
            if local_var.is_captured {
                has_captured_local_var = true;
            }
            // todo: fix it
            if local_var.slot < min_local_var_slot {
                min_local_var_slot = local_var.slot;
            }
        });

        if has_captured_local_var {
            min_local_var_slot as isize + 1
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
        self.local_vars[self.scope_level].insert(name, new_var);
        Ok(slot)
    }

    /// Get name's register number
    fn local_var_slot(&self, name: &String) -> Result<usize> {
        match self.get_current_scope().get(name) {
            Some(local_var) => Ok(local_var.slot),
            _ => Err(Error::IllegalRegister),
        }
    }

    /// Remove a variable from current scope
    #[inline]
    fn remove_local_var(&mut self, name: &String) {
        self.free_register();
        self.get_current_scope_mut().remove(name);
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
        // todo: refactor scope lookup
        if let Some(up_value) = self.up_values.get(name) {
            Ok(up_value.index)
        } else {
            unimplemented!()
        }
    }

    fn close_open_up_values(&mut self, line: Line) {
        let a = self.get_jump_arg_a();
        if a > 0 {
            self.emit_jump(line, a, 0);
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

    fn to_prototypes(sub_fns: &Vec<Rc<FnInfo>>) -> Vec<Rc<Prototype>> {
        sub_fns.iter().map(|sub_fn| {
            sub_fn.clone().to_prototype()
        }).collect()
    }

    fn get_local_vars(&self) -> Vec<LocalVar> {
        unimplemented!()
    }

    fn get_up_value_names(&self) -> Vec<String> {
        unimplemented!()
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
            if up_val.up_value_index.is_some() {
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
    }

    #[inline]
    fn emit_ABx(&mut self, line: Line, opcode: u8, a: isize, bx: isize) {
        let ins = bx << 14 | a << 6 | opcode as isize;
        self.instructions.push(ins as u32);
    }

    #[inline]
    fn emit_AsBx(&mut self, line: Line, opcode: u8, a: isize, b: isize) {
        let ins = (b + MAXARG_SBX) << 14 | a << 6 | opcode as isize;
        self.instructions.push(ins as u32);
    }

    #[inline]
    fn emit_Ax(&mut self, line: Line, opcode: u8, ax: isize) {
        let ins = ax << 6 | opcode as isize;
        self.instructions.push(ins as u32);
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
        unimplemented!()
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
    fn emit_jump(&mut self, line: Line, a: isize, sBx: isize) -> usize {
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

    #[inline]
    fn emit_for_prep(&mut self, line: Line, a: isize, sBx: isize) -> isize {
        self.emit_AsBx(line, opcode::OP_FORPREP, a, sBx);
        self.instructions.len() as isize - 1
    }

    #[inline]
    fn emit_for_loop(&mut self, line: Line, a: isize, sBx: isize) -> isize {
        self.emit_AsBx(line, opcode::OP_FORLOOP, a, sBx);
        self.instructions.len() as isize - 1
    }

    // r(a+3), ... ,r(a+2+c) := r(a)(r(a+1), r(a+2));
    #[inline]
    fn emit_t_for_call(&mut self, line: Line, a: isize, c: isize) {
        // todo
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
    // arith & bitwise & relational
    fn emit_binary_op(&mut self, line: Line, op: &Token, a: isize, b: isize, c: isize) {
        match op {
            Token::OpAdd => self.emit_ABC(line, opcode::OP_ADD, a, b, c),
            Token::OpMinus => self.emit_ABC(line, opcode::OP_SUB, a, b, c),
            Token::OpMul => self.emit_ABC(line, opcode::OP_MUL, a, b, c),
            Token::OpMod => self.emit_ABC(line, opcode::OP_MOD, a, b, c),
            Token::OpPow => self.emit_ABC(line, opcode::OP_POW, a, b, c),
            Token::OpDiv => self.emit_ABC(line, opcode::OP_DIV, a, b, c),
            Token::OpIDiv => self.emit_ABC(line, opcode::OP_IDIV, a, b, c),
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

                self.emit_jump(line, 0, 1);
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
        self.codegen_ret_stat(&block.ret_exps, block.last_line)
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
            let is_mult_ret = is_vararg_or_fn_call(exps.last().unwrap());
            let num = exps.len() - 1;
            for (i, exp) in exps.iter().enumerate() {
                let reg = self.alloc_register()? as isize;
                // has `...` or function call
                if i == num && is_mult_ret {
                    self.codegen_exp(exp, reg, -1);
                } else {
                    self.codegen_exp(exp, reg, 1);
                }
            }
            self.free_registers(num);
            let a = self.used_regs;
            if is_mult_ret {
                self.emit_return(last_line, a as isize, -1);
            } else {
                self.emit_return(last_line, a as isize, num as isize);
            }

            Ok(())
        }
    }

    // local function f() end => local f; f = function() end
    fn codegen_local_fn_def_stat(&mut self, name: &String, fn_def: &FnDef) -> Result<()> {
        let reg = self.add_local_var(name.clone())?;
        unimplemented!()
    }

    fn codegen_fn_call_stat(&mut self, fn_call: &FnCall) -> Result<()> {
        unimplemented!()
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


    /*
            ______________
           |  false? jmp  |
           V              /
    repeat block until exp
    */
    fn codegen_repeat_stat(&mut self, exp: &Exp, block: &Block) -> Result<()> {
        unimplemented!()
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
        let reg = self.alloc_register();
        unimplemented!()
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
    fn codegen_condition_stat(&mut self, exp: &Vec<Exp>, block: &Vec<Block>) -> Result<()> {
        unimplemented!()
    }

    fn codegen_for_num_stat(&mut self, for_num: &ForNum) -> Result<()> {
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
    fn codegen_exp(&mut self, exp: &Exp, a: isize, n: isize) {
        match exp {
            Exp::Nil(line) => self.emit_load_nil(*line, a, n),
            Exp::False(line) => self.emit_load_bool(*line, a, 0, 0),
            Exp::True(line) => self.emit_load_bool(*line, a, 1, 0),
            Exp::Integer(num, line) => self.emit_load_k(*line, a, Constant::Integer(*num)),
            Exp::Float(num, line) => self.emit_load_k(*line, a, Constant::Number(*num)),
            Exp::String(s, line) => self.emit_load_k(*line, a, Constant::String(s.clone())),
            Exp::Name(name, line) => { self.codegen_name_exp(name, a, n, *line); }
            Exp::Parens(exp) => self.codegen_exp(&*exp, a, n),
            Exp::Vararg(line) => { self.codegen_vararg_exp(a, n, *line); }
            Exp::Unop(op, exp, line) => {}
            Exp::Binop(exp1, op, exp2, line) => { self.codegen_binop_exp(&*exp1, op, &*exp2, a, n, *line); }
            Exp::Concat(exps, line) => { self.codegen_concat_exp(exps, a, n, *line); }
            Exp::TableConstructor(fields, line) => { self.codegen_table_constructor_exp(fields, a, n, *line); }
            Exp::TableAccess(obj, key, line) => { self.codegen_table_access_exp(&*obj, &*key, a, n, *line); }
            Exp::FnDef(fn_def) => { self.codegen_fn_def_exp(fn_def, a); }
            Exp::FnCall(fn_call) => { self.codegen_fn_call_exp(fn_call, a, n); }
        }
    }

    fn codegen_name_exp(&mut self, name: &String, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_fn_def_exp(&mut self, fn_def: &FnDef, a: isize) -> Result<()> {
        unimplemented!()
    }

    fn codegen_vararg_exp(&mut self, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_unop_exp(&mut self, op: &Token, exp: &Exp, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_binop_exp(&mut self, exp1: &Exp, op: &Token, exp2: &Exp, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_concat_exp(&mut self, exps: &Vec<Exp>, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_table_constructor_exp(&mut self, fields: &Vec<Field>, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_table_access_exp(&mut self, obj: &Exp, key: &Exp, a: isize, n: isize, line: Line) -> Result<()> {
        unimplemented!()
    }

    fn codegen_fn_call_exp(&mut self, fn_call: &FnCall, a: isize, n: isize) -> Result<()> {
        unimplemented!()
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
    use super::*;

    #[test]
    fn test_codegen() {}
}