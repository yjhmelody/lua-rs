#![allow(dead_code)]

use crate::compiler::ast::*;
use crate::compiler::error::*;
use crate::compiler::lexer::*;
use crate::compiler::token::Token;

fn parse_block(lexer: &mut Lexer) -> Result<Block> {
    Ok(Block::new(
        parse_stats(lexer)?,
        parse_ret_exps(lexer)?,
        lexer.current_line(),
    ))
}

fn parse_stats(lexer: &mut Lexer) -> Result<Vec<Stat>> {
    let mut stats = vec![];
    while !is_return_or_block_end(lexer.look_ahead()) {
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
    match lexer.look_ahead()? {
        Token::Eof | Token::KwElse | Token::KwElseIf | Token::KwEnd | Token::KwUntil => Ok(vec![]),
        Token::SepSemi => {
            lexer.next_token()?;
            Ok(vec![])
        }
        _ => {
            let exps = parse_exp_list(lexer);
            match lexer.look_ahead() {
                Ok(Token::SepSemi) => {
                    lexer.next_token()?;
                }
                _ => {}
            };

            exps
        }
    }
}

fn parse_exp_list(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    let mut exp_list = vec![];
    exp_list.push(parse_exp(lexer)?);
    while let Ok(Token::SepSemi) = lexer.look_ahead() {
        lexer.next_token()?;
        exp_list.push(parse_exp(lexer)?);
    }

    Ok(exp_list)
}

fn parse_exp(lexer: &mut Lexer) -> Result<Exp> {
    unimplemented!()
}

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
        match name {
            Token::Identifier(name) => Ok(Stat::Label { name }),
            _ => unreachable!(),
        }
    }
}

fn parse_goto_stat(lexer: &mut Lexer) -> Result<Stat> {
    // skip `goto`
    lexer.next_token()?;
    match lexer.next_ident()? {
        Token::Identifier(name) => Ok(Stat::Goto { name }),
        _ => unreachable!(),
    }
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
    let name = match lexer.next_ident() {
        Ok(Token::Identifier(s)) => s,
        _ => {
            return Err(Error::NotIdentifier);
        }
    };
    if let Ok(Token::OpAssign) = lexer.look_ahead() {
        // =
        _parse_for_num_stat(lexer, line_of_for, name)
    } else {
        // in
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

    // optinal exp, default to 1
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
    while let Ok(Token::SepComma) = lexer.next_token() {
        let name = match lexer.next_ident() {
            Ok(Token::Identifier(s)) => s,
            err => {
                return Err(Error::NotIdentifier);
            }
        };
        name_list.push(name);
    }

    Ok(name_list)
}

fn parse_local_assign_or_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn prse_assign_or_fn_call_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_assign_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_fn_def_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn is_return_or_block_end(tok: Result<Token>) -> bool {
    match tok {
        Ok(tok) => match tok {
            Token::KwReturn | Token::KwEnd | Token::KwElse | Token::KwElseIf | Token::KwUntil => {
                true
            }
            _ => false,
        },
        _ => is_err_eof(tok),
    }
}

// 将EOF Error 转为EOF Token
fn err_eof_to_token(tok: Result<Token>) -> Result<Token> {
    match tok {
        Err(Error::EOF) => Ok(Token::Eof),
        tok => tok,
    }
}

fn is_err_eof(tok: Result<Token>) -> bool {
    match tok {
        Err(Error::EOF) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {}
}
