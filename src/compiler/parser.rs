#![allow(dead_code)]

use std::collections::HashMap;

use crate::compiler::ast::*;
use crate::compiler::error::*;
use crate::compiler::lexer::*;
use crate::compiler::token::Token;

fn parse(lexer: &mut Lexer) -> Block {
    // todo: handler all errors in this function?
    unimplemented!()
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
    match lexer.look_ahead()? {
        Token::KwReturn => {}
        _ => return Ok(vec![]),
    };
    // skip `return`
    lexer.next_token()?;
    match lexer.look_ahead() {
        Err(Error::EOF) | Ok(Token::KwElse) | Ok(Token::KwElseIf) | Ok(Token::KwEnd) | Ok(Token::KwUntil) => Ok(vec![]),
        Ok(Token::SepSemi) => {
            lexer.next_token()?;
            Ok(vec![])
        }
        _ => {
            let exps = parse_exp_list(lexer);
            if let Ok(Token::SepSemi) = lexer.look_ahead() {
                lexer.next_token()?;
            }
            exps
        }
    }
}

fn parse_exp_list(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    let mut exp_list = vec![];
    exp_list.push(parse_exp(lexer)?);
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.next_token()?;
        exp_list.push(parse_exp(lexer)?);
    }

    Ok(exp_list)
}

/********************* Parse stat  **********************/

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
        _ => unimplemented!(),
    }
}

fn parse_empty_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.next_token()?;
    Ok(Stat::Empty)
}

fn parse_break_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.next_token()?;
    Ok(Stat::Break {
        line: lexer.current_line(),
    })
}

fn parse_label_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `::`
    lexer.next_token()?;
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
    lexer.next_token()?;
    let name = lexer.next_ident()?;
    Ok(Stat::Goto { name })
}

fn parse_do_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `do`
    lexer.next_token()?;
    let block = Box::new(parse_block(lexer)?);
    match lexer.next_token() {
        Ok(Token::KwEnd) => Ok(Stat::Do { block: block }),
        _ => Err(Error::IllegalStat),
    }
}

fn parse_while_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.next_token()?;
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
    lexer.next_token()?;
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
    lexer.next_token()?;
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
        lexer.next_token()?;
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
        lexer.next_token()?;
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
    lexer.next_token()?;
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
    lexer.next_token()?;
    let init_exp = parse_exp(lexer)?;
    let limit_exp = match lexer.look_ahead() {
        Ok(Token::SepComma) => {
            lexer.next_token()?;
            parse_exp(lexer)?
        }
        _ => {
            return Err(Error::IllegalStat);
        }
    };

    // optional exp, default to 1
    let step_exp = match lexer.look_ahead() {
        Ok(Token::SepComma) => {
            lexer.next_token()?;
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
    lexer.next_token()?;
    match lexer.look_ahead() {
        Ok(Token::KwFunction) => _parse_local_fn_def_stat(lexer),
        _ => _parse_local_var_decl_stat(lexer),
    }
}

fn _parse_local_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    lexer.next_token()?;
    let name = lexer.next_ident()?;
    let exp = parse_fn_def_exp(lexer)?;
    Ok(Stat::LocalFnDef { name, exp })
}

fn _parse_local_var_decl_stat(lexer: &mut Lexer) -> Result<Stat> {
    let name0 = lexer.next_ident()?;
    let name_list = _parse_name_list(lexer, name0)?;
    let exp_list = if let Ok(Token::OpAssign) = lexer.look_ahead() {
        lexer.next_token()?;
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
    let prefix_exp = parse_prefix_exp(lexer)?;
    match prefix_exp {
        Exp::FnCall {
            line,
            last_line,
            prefix_exp,
            name_exp,
            args,
        } => {
            Ok(Stat::FnCall {
                line,
                last_line,
                prefix_exp,
                name_exp,
                args,
            })
        }
        _ => { parse_assign_stat(lexer, prefix_exp) }
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
    lexer.next_token()?;
    let mut has_colon = false;
    let fn_name = _parse_fn_name(lexer, &mut has_colon)?;
    let mut fn_body = parse_fn_def_exp(lexer)?;
    // v:name(args) => v.name(self, args)
    // insert `self` to the first arg
    // todo: refactor
    match fn_body {
        Exp::FnDef {
            line,
            last_line,
            ref mut par_list,
            is_vararg,
            ref block,
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
    if _check_var(&var0) {
        var_list.push(var0);
    } else {
        return Err(Error::NotVarExpression);
    }
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.next_token()?;
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
        lexer.next_token()?;
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
        lexer.next_token()?;
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
    unimplemented!()
}

fn parse_exp10(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

fn parse_exp9(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

fn parse_exp8(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

fn parse_exp7(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

fn parse_exp6(lexer: &mut Lexer) -> Result<Exp> {
    let exp = parse_exp5(lexer)?;
//    match lexer.look_ahead()? {
//    }
    unimplemented!()
}

fn parse_exp5(lexer: &mut Lexer) -> Result<Exp> {
    let exp = parse_exp4(lexer)?;
    match lexer.look_ahead()? {
        Token::OpConcat => {
            let mut line = 0;
            let mut exps = vec![];

            while let Token::OpConcat = lexer.look_ahead()? {
                lexer.next_token()?;
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
    unimplemented!()
}

fn parse_exp3(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

fn parse_exp2(lexer: &mut Lexer) -> Result<Exp> {
    // parse unary ops
    match lexer.look_ahead()? {
        Token::OpNot | Token::OpLen | Token::OpWave | Token::OpMinus => {
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
    match lexer.look_ahead()? {
        Token::VarArg => {
            lexer.next_token()?;
            let line = lexer.current_line();
            Ok(Exp::Vararg { line })
        }
        Token::KwNil => {
            lexer.next_token()?;
            let line = lexer.current_line();
            Ok(Exp::Nil { line })
        }
        Token::KwTrue => {
            lexer.next_token()?;
            let line = lexer.current_line();
            Ok(Exp::True { line })
        }
        Token::KwFalse => {
            lexer.next_token()?;
            let line = lexer.current_line();
            Ok(Exp::False { line })
        }
        Token::String(val) => {
            lexer.next_token()?;
            let line = lexer.current_line();
            Ok(Exp::String { line, val })
        }
        Token::Number(_) => parse_number_exp(lexer),

        // followings are recursive
        Token::SepLcurly => parse_table_constructor_exp(lexer),
        Token::KwFunction => parse_fn_def_exp(lexer),
        _ => parse_prefix_exp(lexer),
    }
}

fn parse_number_exp(lexer: &mut Lexer) -> Result<Exp> {
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
    unimplemented!()
}

fn _parse_field_list(lexer: &mut Lexer) -> Result<(Vec<Exp>, Vec<Exp>)> {
    let mut key_exps = vec![];
    let mut val_exps = vec![];
    if let Token::SepRcurly = lexer.look_ahead()? {
        return Ok((key_exps, val_exps));
    }

    let (k, v) = _parse_field(lexer)?;
    key_exps.push(k);
    val_exps.push(v);

    while _is_field_sep(lexer.look_ahead()?) {
        lexer.next_token()?;
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

fn _parse_field(lexer: &mut Lexer) -> Result<(Exp, Exp)> {
    // field ::= `[` exp `]` `=` exp | Name `=` exp | exp
    unimplemented!()
}

fn _parse_par_list(lexer: &mut Lexer, is_vararg: &mut bool) -> Result<Vec<String>> {
    let mut params = vec![];
    match lexer.look_ahead()? {
        Token::SepRparen => { return Ok(params); }
        Token::VarArg => {
            lexer.next_token()?;
            *is_vararg = true;
            return Ok(params);
        }
        _ => {}
    }

    params.push(lexer.next_ident()?);
    while let Ok(Token::SepComma) = lexer.look_ahead() {
        lexer.next_token()?;
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
fn _check_var(exp: &Exp) -> bool {
    match exp {
        Exp::Name { line, val } => true,
        Exp::TableAccess { last_line, prefix_exp, key_exp } => true,
        _ => false,
    }
}

#[inline]
fn _is_field_sep(tok: Token) -> bool {
    match tok {
        Token::SepComma | Token::SepSemi => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[test]
    fn test_parser() {}
}
