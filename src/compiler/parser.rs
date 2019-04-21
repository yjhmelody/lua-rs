#![allow(dead_code)]

use crate::compiler::ast::*;
use crate::compiler::error::*;
use crate::compiler::lexer::*;
use crate::compiler::token::Token;

pub fn parse(lexer: &mut Lexer) {
    // todo: handler all errors in this function?
    parse_block(lexer);
}

fn parse_block(lexer: &mut Lexer) -> Result<Block> {
    Ok(Block::new(
        parse_stats(lexer)?,
        parse_ret_exps(lexer)?,
        lexer.current_line(),
    ))
}

fn parse_stats(lexer: &mut Lexer) -> Result<Vec<Stat>> {

    let mut stats = vec![];
    while !_is_return_or_block_end(lexer.look_ahead()) {

        let stat = parse_stat(lexer)?;
        match stat {
            Stat::Empty => {}
            stat => {
                stats.push(stat);
            }
        }
    }
    Ok(stats)
}

fn parse_ret_exps(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    match lexer.look_ahead() {
        Ok(Token::KwReturn) => {}
        _ => return Ok(vec![]),
    };
    // skip `return`
    lexer.skip_next_token();
    match lexer.look_ahead() {
        Err(Error::EOF) | Ok(Token::KwElse) | Ok(Token::KwElseIf) | Ok(Token::KwEnd) | Ok(Token::KwUntil) => Ok(vec![]),
        Ok(Token::SepSemi) => {
            lexer.skip_next_token();
            Ok(vec![])
        }
        _ => {
            let exps = parse_exp_list(lexer);
            if let Ok(Token::SepSemi) = lexer.look_ahead() {
                lexer.skip_next_token();
            }
            exps
        }
    }
}

fn parse_exp_list(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    // exp {, exp}
    let mut exp_list = vec![];
    exp_list.push(parse_exp(lexer)?);
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.skip_next_token();
        exp_list.push(parse_exp(lexer)?);
    }

    Ok(exp_list)
}

/********************* Parse Statement  **********************/

fn parse_stat(lexer: &mut Lexer) -> Result<Stat> {
    match lexer.look_ahead()? {
        // deal with `;`
        Token::SepSemi => parse_empty_stat(lexer),
        Token::KwBreak => parse_break_stat(lexer),
        Token::SepLabel => parse_label_stat(lexer),
        Token::KwGoto => parse_goto_stat(lexer),
        Token::KwDo => parse_do_stat(lexer),
        Token::KwWhile => parse_while_stat(lexer),
        Token::KwIf => parse_if_stat(lexer),
        Token::KwRepeat => parse_repeat_stat(lexer),
        Token::KwFor => parse_for_stat(lexer),
        Token::KwFunction => parse_fn_def_stat(lexer),
        Token::KwLocal => parse_local_assign_or_fn_def_stat(lexer),
        _ => parse_assign_or_fn_call_stat(lexer),
    }
}

fn parse_empty_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    Ok(Stat::Empty)
}

fn parse_break_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    Ok(Stat::Break {
        line: lexer.current_line(),
    })
}

fn parse_label_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `::`
    lexer.skip_next_token();
    let name = lexer.next_ident()?;
    // check `::`
    let tok = lexer.next_token()?;
    if tok != Token::SepLabel {
        Err(Error::IllegalStat)
    } else {
        Ok(Stat::Label { name })
    }
}

fn parse_goto_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `goto`
    lexer.skip_next_token();
    let name = lexer.next_ident()?;
    Ok(Stat::Goto { name })
}

fn parse_do_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `do`
    lexer.skip_next_token();
    let block = Box::new(parse_block(lexer)?);
    match lexer.next_token() {
        Ok(Token::KwEnd) => Ok(Stat::Do { block: block }),
        _ => Err(Error::IllegalStat),
    }
}

fn parse_while_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    let exp = parse_exp(lexer)?;
    match lexer.next_token() {
        Ok(Token::KwDo) => {
            let block = Box::new(parse_block(lexer)?);
            let end = lexer.next_token()?;
            if end != Token::KwEnd {
                Err(Error::IllegalStat)
            } else {
                Ok(Stat::While { exp, block })
            }
        }
        _ => Err(Error::IllegalStat),
    }
}

fn parse_repeat_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `repeat`
    lexer.skip_next_token();
    let block = Box::new(parse_block(lexer)?);
    match lexer.next_token() {
        Ok(Token::KwUntil) => {
            let exp = parse_exp(lexer)?;
            Ok(Stat::Repeat { exp, block })
        }
        _ => Err(Error::IllegalStat),
    }
}

fn parse_if_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `if`
    lexer.skip_next_token();
    let mut exps = vec![];
    let mut blocks = vec![];
    exps.push(parse_exp(lexer)?);
    // skip `then`
    match lexer.next_token() {
        Ok(Token::KwThen) => {
            blocks.push(parse_block(lexer)?);
        }
        _ => {
            return Err(Error::IllegalStat);
        }
    }

    // elseif
    while let Ok(Token::KwElseIf) = lexer.look_ahead() {
        lexer.skip_next_token();
        exps.push(parse_exp(lexer)?);

        match lexer.next_token() {
            Ok(Token::KwThen) => {
                blocks.push(parse_block(lexer)?);
            }
            _ => {
                return Err(Error::IllegalStat);
            }
        };
    }

    // else
    if let Ok(Token::KwElse) = lexer.look_ahead() {
        lexer.skip_next_token();
        // todo
        exps.push(parse_exp(lexer)?);
        match lexer.next_token() {
            Ok(Token::KwThen) => {
                blocks.push(parse_block(lexer)?);
            }
            _ => {
                return Err(Error::IllegalStat);
            }
        };
    }

    Ok(Stat::If { exps, blocks })
}

fn parse_for_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    let line_of_for = lexer.current_line();
    let name = lexer.next_ident()?;
    if let Ok(Token::OpAssign) = lexer.look_ahead() {
        // `=`
        _parse_for_num_stat(lexer, line_of_for, name)
    } else {
        // `in`
        _parse_for_in_stat(lexer, name)
    }
}

fn _parse_for_num_stat(lexer: &mut Lexer, line_of_for: Line, var_name: String) -> Result<Stat> {
    lexer.skip_next_token();
    let init_exp = parse_exp(lexer)?;
    let limit_exp = match lexer.look_ahead() {
        Ok(Token::SepComma) => {
            lexer.skip_next_token();
            parse_exp(lexer)?
        }
        _ => {
            return Err(Error::IllegalStat);
        }
    };

    // optional exp, default to 1
    let step_exp = match lexer.look_ahead() {
        Ok(Token::SepComma) => {
            lexer.skip_next_token();
            parse_exp(lexer)?
        }
        _ => Exp::Integer {
            line: lexer.current_line(),
            val: 1,
        },
    };

    let line_of_do = match lexer.next_token() {
        Ok(Token::KwDo) => lexer.current_line(),
        _ => {
            return Err(Error::IllegalStat);
        }
    };

    let block = Box::new(parse_block(lexer)?);
    match lexer.next_token() {
        Ok(Token::KwEnd) => {}
        _ => {
            return Err(Error::IllegalStat);
        }
    };

    Ok(Stat::ForNum {
        line_of_for,
        line_of_do,
        var_name,
        exps: (init_exp, limit_exp, step_exp),
        block,
    })
}

fn parse_local_assign_or_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    match lexer.look_ahead() {
        Ok(Token::KwFunction) => _parse_local_fn_def_stat(lexer),
        _ => _parse_local_var_decl_stat(lexer),
    }
}

fn _parse_local_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.skip_next_token();
    let name = lexer.next_ident()?;
    let exp = parse_fn_def_exp(lexer)?;
    Ok(Stat::LocalFnDef { name, exp })
}

fn _parse_local_var_decl_stat(lexer: &mut Lexer) -> Result<Stat> {
    let name0 = lexer.next_ident()?;
    let name_list = _parse_name_list(lexer, name0)?;
    let exp_list = if let Ok(Token::OpAssign) = lexer.look_ahead() {
        lexer.skip_next_token();
        parse_exp_list(lexer)?
    } else {
        vec![]
    };
    let last_line = lexer.current_line();
    Ok(Stat::LocalVarDecl {
        last_line,
        name_list,
        exp_list,
    })
}

fn parse_assign_or_fn_call_stat(lexer: &mut Lexer) -> Result<Stat> {
    let prefix_exp = parse_prefix_exp(lexer);

    match prefix_exp {
        Ok(Exp::FnCall {
            line,
            last_line,
            prefix_exp,
            name_exp,
            args,
           }) => {
            Ok(Stat::FnCall {
                line,
                last_line,
                prefix_exp,
                name_exp,
                args,
            })
        }
        _ => { parse_assign_stat(lexer, prefix_exp.unwrap()) }
    }
}

fn parse_assign_stat(lexer: &mut Lexer, var0: Exp) -> Result<Stat> {
    let var_list = _parse_var_list(lexer, var0)?;
    match lexer.next_token() {
        Ok(Token::OpAssign) => {
            let exp_list = parse_exp_list(lexer)?;
            let last_line = lexer.current_line();
            Ok(Stat::Assign {
                last_line,
                var_list,
                exp_list,
            })
        }
        _ => { Err(Error::IllegalStat) }
    }
}

fn parse_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `function`
    lexer.skip_next_token();
    let mut has_colon = false;
    let fn_name = _parse_fn_name(lexer, &mut has_colon)?;
    let mut fn_body = parse_fn_def_exp(lexer)?;
    // v:name(args) => v.name(self, args)
    // insert `self` to the first arg
    // todo: refactor
    match fn_body {
        Exp::FnDef {
            line: _,
            last_line,
            ref mut par_list,
            is_vararg: _,
            block: _,
        } => {
            if has_colon {
                par_list.reverse();
                par_list.push("self".to_string());
                par_list.reverse();
            }
            // transfer function definition to assignment
            Ok(Stat::Assign {
                last_line,
                var_list: vec![fn_name],
                exp_list: vec![fn_body],
            })
        }
        _ => unreachable!(),
    }
}

fn _parse_var_list(lexer: &mut Lexer, var0: Exp) -> Result<Vec<Exp>> {
    let mut var_list = vec![];
    if _is_var_exp(&var0) {
        var_list.push(var0);
    } else {
        return Err(Error::NotVarExpression);
    }
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.skip_next_token();
        let exp = parse_prefix_exp(lexer)?;
        var_list.push(exp);
    }
    Ok(var_list)
}

fn _parse_for_in_stat(lexer: &mut Lexer, name: String) -> Result<Stat> {
    let name_list = _parse_name_list(lexer, name)?;
    match lexer.next_token() {
        Ok(Token::KwIn) => {
            let exp_list = parse_exp_list(lexer)?;
            let line_of_do = match lexer.next_token() {
                Ok(Token::KwDo) => lexer.current_line(),
                _ => {
                    return Err(Error::IllegalStat);
                }
            };
            let block = Box::new(parse_block(lexer)?);
            match lexer.next_token() {
                Ok(Token::KwEnd) => Ok(Stat::ForIn {
                    line_of_do,
                    name_list,
                    exp_list,
                    block,
                }),
                _ => Err(Error::IllegalStat),
            }
        }

        _ => Err(Error::IllegalStat),
    }
}

fn _parse_name_list(lexer: &mut Lexer, name0: String) -> Result<Vec<String>> {
    let mut name_list = vec![name0];
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        let name = lexer.next_ident()?;
        name_list.push(name);
    }

    Ok(name_list)
}

fn _parse_fn_name(lexer: &mut Lexer, has_colon: &mut bool) -> Result<Exp> {
    // fn_name ::= Name {`.` Name} [`:` Name]
    let name = lexer.next_ident()?;
    let line = lexer.current_line();
    let mut exp = Box::new(Exp::Name { line, val: name });

    while let Ok(Token::SepDot) = lexer.look_ahead() {
        lexer.skip_next_token();
        let name = lexer.next_ident()?;
        let line = lexer.current_line();
        let idx = Box::new(Exp::String { line, val: name });
        exp = Box::new(Exp::TableAccess {
            last_line: line,
            prefix_exp: exp,
            key_exp: idx,
        });
    }

    // check `:`
    if let Ok(Token::SepColon) = lexer.look_ahead() {
        lexer.skip_next_token();
        let name = lexer.next_ident()?;
        let line = lexer.current_line();
        *has_colon = true;
        let idx = Box::new(Exp::String { line, val: name });
        exp = Box::new(Exp::TableAccess {
            last_line: line,
            prefix_exp: exp,
            key_exp: idx,
        })
    }

    Ok(*exp)
}

/******************* Parse Expression *************************/

fn parse_exp(lexer: &mut Lexer) -> Result<Exp> {
    parse_exp12(lexer)
}

fn parse_exp12(lexer: &mut Lexer) -> Result<Exp> {
    // x or y
    let mut exp = Box::new(parse_exp11(lexer)?);

    while let Ok(Token::OpOr) = lexer.look_ahead() {
        let op = lexer.next_token().or(Err(Error::NotOperator))?;
        let line = lexer.current_line();

        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp11(lexer)?),
        });
    }

    Ok(*exp)
}

fn parse_exp11(lexer: &mut Lexer) -> Result<Exp> {
    // x and y
    let mut exp = Box::new(parse_exp10(lexer)?);
    while let Ok(Token::OpAnd) = lexer.look_ahead() {
        let op = lexer.next_token()?;
        let line = lexer.current_line();
        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp10(lexer)?),
        });
    }
    Ok(*exp)
}

fn parse_exp10(lexer: &mut Lexer) -> Result<Exp> {
    // x `cmp` y
    let mut exp = Box::new(parse_exp9(lexer)?);
    loop {
        match lexer.look_ahead() {
            Ok(Token::OpGe) | Ok(Token::OpGt) | Ok(Token::OpLe) | Ok(Token::OpLt) | Ok(Token::OpNe) | Ok(Token::OPEq) => {
                let op = lexer.next_token()?;
                let line = lexer.current_line();
                exp = Box::new(Exp::Binop {
                    line,
                    op,
                    exp1: exp,
                    exp2: Box::new(parse_exp9(lexer)?),
                });
            }
            _ => { break; }
        }
    }

    Ok(*exp)
}

fn parse_exp9(lexer: &mut Lexer) -> Result<Exp> {
    // x | y
    let mut exp = Box::new(parse_exp8(lexer)?);
    while let Ok(Token::OpBitOr) = lexer.look_ahead() {
        let op = lexer.next_token()?;
        let line = lexer.current_line();
        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp8(lexer)?),
        });
    }
    Ok(*exp)
}

fn parse_exp8(lexer: &mut Lexer) -> Result<Exp> {
    // x ~ y
    let mut exp = Box::new(parse_exp7(lexer)?);
    while let Ok(Token::OpWave) = lexer.look_ahead() {
        let op = lexer.next_token()?;
        let line = lexer.current_line();
        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp7(lexer)?),
        });
    }
    Ok(*exp)
}

fn parse_exp7(lexer: &mut Lexer) -> Result<Exp> {
    // x & y
    let mut exp = Box::new(parse_exp6(lexer)?);
    while let Ok(Token::OpBitAnd) = lexer.look_ahead() {
        let op = lexer.next_token()?;
        let line = lexer.current_line();
        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp6(lexer)?),
        });
    }
    Ok(*exp)
}

fn parse_exp6(lexer: &mut Lexer) -> Result<Exp> {
    // x >>/<< y
    let mut exp = Box::new(parse_exp5(lexer)?);
    loop {
        match lexer.look_ahead() {
            Ok(Token::OpShl) | Ok(Token::OpShr) => {
                let op = lexer.next_token()?;
                let line = lexer.current_line();
                exp = Box::new(Exp::Binop {
                    line,
                    op,
                    exp1: exp,
                    exp2: Box::new(parse_exp5(lexer)?),
                });
            }
            _ => { break; }
        }
    }

    Ok(*exp)
}

fn parse_exp5(lexer: &mut Lexer) -> Result<Exp> {
    // x .. y
    let exp = parse_exp4(lexer)?;
    match lexer.look_ahead() {
        Ok(Token::OpConcat) => {
            let mut line = 0;
            let mut exps = vec![];

            while let Ok(Token::OpConcat) = lexer.look_ahead() {
                lexer.skip_next_token();
                line = lexer.current_line();
                exps.push(parse_exp4(lexer)?);
            }

            Ok(Exp::Concat {
                line,
                exps,
            })
        }
        _ => { Ok(exp) }
    }
}

fn parse_exp4(lexer: &mut Lexer) -> Result<Exp> {
    // x +/- y
    let mut exp = Box::new(parse_exp3(lexer)?);
    loop {
        match lexer.look_ahead() {
            Ok(Token::OpAdd) | Ok(Token::OpMinus) => {
                let op = lexer.next_token()?;
                let line = lexer.current_line();
                exp = Box::new(Exp::Binop {
                    line,
                    op,
                    exp1: exp,
                    exp2: Box::new(parse_exp3(lexer)?),
                });
            }
            _ => { break; }
        }
    }

    Ok(*exp)
}

fn parse_exp3(lexer: &mut Lexer) -> Result<Exp> {
    // *  %  /  //
    let mut exp = Box::new(parse_exp2(lexer)?);
    loop {
        match lexer.look_ahead() {
            Ok(Token::OpMul) | Ok(Token::OpDiv) | Ok(Token::OpIDiv) | Ok(Token::OpMod) => {
                let op = lexer.next_token()?;
                let line = lexer.current_line();
                exp = Box::new(Exp::Binop {
                    line,
                    op,
                    exp1: exp,
                    exp2: Box::new(parse_exp2(lexer)?),
                });
            }
            _ => { break; }
        }
    }

    Ok(*exp)
}

fn parse_exp2(lexer: &mut Lexer) -> Result<Exp> {
    // unary ops: not # - ~
    match lexer.look_ahead() {
        Ok(Token::OpNot) | Ok(Token::OpLen) | Ok(Token::OpWave) | Ok(Token::OpMinus) => {
            let op = lexer.next_token()?;
            let line = lexer.current_line();
            Ok(
                Exp::Unop {
                    line,
                    op,
                    exp: Box::new(parse_exp2(lexer)?),
                }
            )
        }
        _ => Ok(parse_exp1(lexer)?),
    }
}

fn parse_exp1(lexer: &mut Lexer) -> Result<Exp> {
    // x ^ y
    let mut exp = Box::new(parse_exp0(lexer)?);
    if let Ok(Token::OpPow) = lexer.look_ahead() {
        let op = lexer.next_token().or(Err(Error::NotOperator))?;
        let line = lexer.current_line();
        exp = Box::new(Exp::Binop {
            line,
            op,
            exp1: exp,
            exp2: Box::new(parse_exp2(lexer)?),
        });
    }
    Ok(*exp)
}

fn parse_exp0(lexer: &mut Lexer) -> Result<Exp> {

    // primary
    match lexer.look_ahead() {
        Ok(Token::VarArg) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(Exp::Vararg { line })
        }
        Ok(Token::KwNil) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(Exp::Nil { line })
        }
        Ok(Token::KwTrue) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(Exp::True { line })
        }
        Ok(Token::KwFalse) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(Exp::False { line })
        }
        Ok(Token::String(val)) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(Exp::String { line, val })
        }
        Ok(Token::Number(_)) => parse_number_exp(lexer),

        // followings are recursive
        Ok(Token::SepLcurly) => parse_table_constructor_exp(lexer),
        Ok(Token::KwFunction) => parse_fn_def_exp(lexer),
        _ => parse_prefix_exp(lexer),
    }
}

/******************* Parse Primary *************************/

fn parse_number_exp(lexer: &mut Lexer) -> Result<Exp> {
    // todo: impl number parser
    unimplemented!()
}

fn parse_table_constructor_exp(lexer: &mut Lexer) -> Result<Exp> {
    let line = lexer.current_line();
    // `{`
    if !lexer.check_next_token(Token::SepLcurly) {
        return Err(Error::IllegalExpression);
    }
    // [fieldlist]
    let (key_exps, val_exps) = _parse_field_list(lexer)?;

    // `}`
    if !lexer.check_next_token(Token::SepRcurly) {
        return Err(Error::IllegalExpression);
    }

    let last_line = lexer.current_line();
    Ok(Exp::TableConstructor {
        line,
        last_line,
        key_exps,
        val_exps,
    })
}

fn parse_fn_def_exp(lexer: &mut Lexer) -> Result<Exp> {
    // it has skip `function` keyword
    let line = lexer.current_line();
    if !lexer.check_next_token(Token::SepLparen) {
        return Err(Error::IllegalToken);
    }
    let mut is_vararg = false;
    let par_list = _parse_par_list(lexer, &mut is_vararg)?;
    if !lexer.check_next_token(Token::SepRparen) {
        return Err(Error::IllegalToken);
    }
    let block = Box::new(parse_block(lexer)?);
    if !lexer.check_next_token(Token::KwEnd) {
        return Err(Error::IllegalToken);
    }
    let last_line = lexer.current_line();
    Ok(Exp::FnDef {
        line,
        last_line,
        par_list,
        is_vararg,
        block,
    })
}

fn parse_prefix_exp(lexer: &mut Lexer) -> Result<Exp> {
    let exp;
    if let Ok(Token::Identifier(val)) = lexer.look_ahead() {
        lexer.skip_next_token();
        let line = lexer.current_line();
        exp = Exp::Name { line, val };
    } else {
        // `(` exp `)`
        exp = parse_parens_exp(lexer)?;
    }

    let mut exp = Box::new(exp);
    loop {
        match lexer.look_ahead() {
            Ok(Token::SepLbrack) => {
                // `[` exp `]`
                lexer.skip_next_token();
                let key_exp = Box::new(parse_exp(lexer)?);
                if !lexer.check_next_token(Token::SepRbrack) {
                    return Err(Error::NotMatchBrackets);
                }
                let last_line = lexer.current_line();

                exp = Box::new(Exp::TableAccess {
                    last_line,
                    prefix_exp: exp,
                    key_exp,
                })
            }
            Ok(Token::SepDot) => {
                lexer.skip_next_token();
                let name = lexer.next_ident()?;
                let line = lexer.current_line();
                let key_exp = Box::new(Exp::String { line, val: name });

                let last_line = line;
                exp = Box::new(Exp::TableAccess {
                    last_line,
                    prefix_exp: exp,
                    key_exp,
                });
            }
            Ok(Token::SepColon)
            | Ok(Token::SepLparen)
            | Ok(Token::SepLcurly)
            | Ok(Token::String(_)) => {
                // [`:` Name] args
                exp = Box::new(_parse_fn_call_exp(lexer, exp)?);
            }

            _ => { return Ok(*exp); }
        }
    }
}

fn parse_parens_exp(lexer: &mut Lexer) -> Result<Exp> {
    if !lexer.check_next_token(Token::SepLparen) {
        return Err(Error::IllegalExpression);
    }
    let exp = parse_exp(lexer)?;

    if !lexer.check_next_token(Token::SepRparen) {
        return Err(Error::NotMatchBrackets);
    }

    // The semantics of vararg and fn call will be changed by parens
    let exp = match exp {
        exp @ Exp::Vararg {
            line: _
        } => Exp::Parens(Box::new(exp)),

        exp @ Exp::FnCall {
            line: _,
            last_line: _,
            prefix_exp: _,
            name_exp: _,
            args: _,
        } => Exp::Parens(Box::new(exp)),

        exp @ Exp::Name {
            line: _,
            val: _
        } => Exp::Parens(Box::new(exp)),

        exp @ Exp::TableAccess {
            last_line: _,
            prefix_exp: _,
            key_exp: _,
        } => Exp::Parens(Box::new(exp)),

        _ => exp,
    };

    Ok(exp)
}

fn _parse_fn_call_exp(lexer: &mut Lexer, prefix_exp: Box<Exp>) -> Result<Exp> {
    // [`:` Name]
    let name_exp = _parse_fn_name_exp(lexer).ok();
    let line = lexer.current_line();
    // args
    let args = _parse_fn_call_args(lexer)?;
    let last_line = lexer.current_line();

    Ok(Exp::FnCall {
        line,
        last_line,
        prefix_exp,
        name_exp,
        args,
    })
}

fn _parse_fn_name_exp(lexer: &mut Lexer) -> Result<Box<Exp>> {
    if let Ok(Token::SepColon) = lexer.look_ahead() {
        lexer.skip_next_token();
        let val = lexer.next_ident()?;
        let line = lexer.current_line();
        Ok(Box::new(Exp::String {
            line,
            val,
        }))
    } else {
        // just represent a option token
        Err(Error::NoMoreTokens)
    }
}

fn _parse_fn_call_args(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    match lexer.look_ahead() {
        // (arg1, arg2 ...)
        Ok(Token::SepLparen) => {
            lexer.skip_next_token();
            if let Ok(Token::SepRparen) = lexer.look_ahead() {
                lexer.skip_next_token();
                Ok(vec![])
            } else {
                let exp = parse_exp_list(lexer);
                if !lexer.check_next_token(Token::SepRparen) {
                    Err(Error::NotMatchBrackets)
                } else {
                    exp
                }
            }
        }

        // function print_prices(table)
        //   print("The clothes costs " .. table.medium)
        //end
        Ok(Token::SepLcurly) => {
            Ok(vec![parse_table_constructor_exp(lexer)?])
        }

        // LiteralString:  print "2" "3" "3"
        Ok(Token::String(val)) => {
            lexer.skip_next_token();
            let line = lexer.current_line();
            Ok(vec![Exp::String { line, val }])
        }

        _ => {
            Err(Error::IllegalFnCall)
        }
    }
}

fn _parse_field_list(lexer: &mut Lexer) -> Result<(Vec<Option<Exp>>, Vec<Exp>)> {
    let mut key_exps = vec![];
    let mut val_exps = vec![];
    if let Ok(Token::SepRcurly) = lexer.look_ahead() {
        return Ok((key_exps, val_exps));
    }

    let (k, v) = _parse_field(lexer)?;
    key_exps.push(k);
    val_exps.push(v);

    while _is_field_sep(lexer.look_ahead()) {
        lexer.skip_next_token();
        // when meet `}`
        match lexer.look_ahead() {
            Ok(Token::SepRcurly) => {
                break;
            }

            _ => {
                let (k, v) = _parse_field(lexer)?;
                key_exps.push(k);
                val_exps.push(v);
            }
        }
    }

    Ok((key_exps, val_exps))
}

fn _parse_field(lexer: &mut Lexer) -> Result<(Option<Exp>, Exp)> {
    // field ::= `[` exp `]` `=` exp | Name `=` exp | exp
    if let Ok(Token::SepLbrack) = lexer.look_ahead() {
        lexer.skip_next_token();
        let key = parse_exp(lexer)?;
        if !lexer.check_next_token(Token::SepRbrack) {
            return Err(Error::NotMatchBrackets);
        }
        if !lexer.check_next_token(Token::OpAssign) {
            return Err(Error::MissingAssignment);
        }

        let val = parse_exp(lexer)?;
        return Ok((Some(key), val));
    }

    // `key` or `value`
    let exp = parse_exp(lexer)?;
    if let Exp::Name { line, ref val } = exp {
        if let Ok(Token::OpAssign) = lexer.look_ahead() {
            lexer.skip_next_token();
            let key = Exp::String { line, val: val.to_string() };
            let val = parse_exp(lexer)?;
            return Ok((Some(key), val));
        }
    }

    Ok((None, exp))
}

fn _parse_par_list(lexer: &mut Lexer, is_vararg: &mut bool) -> Result<Vec<String>> {
    let mut params = vec![];
    match lexer.look_ahead() {
        Ok(Token::SepRparen) => { return Ok(params); }
        Ok(Token::VarArg) => {
            lexer.skip_next_token();
            *is_vararg = true;
            return Ok(params);
        }
        _ => {}
    }

    params.push(lexer.next_ident()?);
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.skip_next_token();
        match lexer.look_ahead() {
            Ok(Token::Identifier(s)) => {
                params.push(s);
            }
            Ok(Token::VarArg) => {
                *is_vararg = true;
                break;
            }
            _ => {
                return Err(Error::IllegalFunction);
            }
        }
    }
    Ok(params)
}

#[inline]
fn _is_return_or_block_end(tok: Result<Token>) -> bool {
    match tok {
        Err(Error::EOF)
        | Ok(Token::KwReturn)
        | Ok(Token::KwEnd)
        | Ok(Token::KwElse)
        | Ok(Token::KwElseIf)
        | Ok(Token::KwUntil) => true,
        _ => false,
    }
}

#[inline]
fn _is_var_exp(exp: &Exp) -> bool {
    match exp {
        Exp::Name { line: _, val: _ } => true,
        Exp::TableAccess { last_line: _, prefix_exp: _, key_exp: _ } => true,
        _ => false,
    }
}

#[inline]
fn _is_field_sep(tok: Result<Token>) -> bool {
    match tok {
        Ok(Token::SepComma) | Ok(Token::SepSemi) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r##"
        function foo()
          function bar()
          end
        end
        "##.to_string();

        let mut lexer = Lexer::from_iter(s.bytes(), "test".to_string());
        parse(&mut lexer);
    }
}
