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
    let mut exps = vec![];
    let tok = lexer.look_ahead();
    let tok = err_eof_to_token(tok);
    match tok {
        Ok(Token::KwReturn) => {}
        _ => {
            lexer.next_token()?;
            match lexer.look_ahead() {
                Ok(tok) => match tok {
                    Token::Eof
                    | Token::KwEnd
                    | Token::KwElse
                    | Token::KwElseIf
                    | Token::KwUntil => {
                        return Ok(exps);
                    }
                    Token::SepSemi => {
                        lexer.next_token()?;
                        return Ok(exps);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    unimplemented!()
}

fn parse_exp_list(lexer: &mut Lexer) -> Result<Vec<Exp>> {
    unimplemented!()
}

fn parse_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_break_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_label_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_goto_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_do_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_while_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_repeat_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_if_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
}

fn parse_for_stat(lexer: &mut Lexer) -> Result<Stat> {
    unimplemented!()
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
}
