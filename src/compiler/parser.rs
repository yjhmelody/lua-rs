#![allow(dead_code)]

use crate::compiler::ast::*;
use crate::compiler::lexer::*;
use crate::compiler::token::Token;

fn parse_block(lexer: &mut Lexer) -> Block {
    Block::new(
        parse_stats(lexer),
        parse_ret_exps(lexer),
        lexer.current_line(),
    )
}

fn parse_stats(lexer: &mut Lexer) -> Vec<Stat> {
    unimplemented!()
}

fn parse_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_ret_exps(lexer: &mut Lexer) -> Vec<Exp> {
    unimplemented!()
}

fn parse_exp_list(lexer: &mut Lexer) -> Vec<Exp> {
    unimplemented!()
}

fn parse_break_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_label_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_goto_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_do_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_while_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_repeat_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_if_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_for_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_local_assign_or_fn_def_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_assign_or_fn_call_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_assign_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn parse_fn_def_stat(lexer: &mut Lexer) -> Stat {
    unimplemented!()
}

fn is_return_or_block_end(tok: Token) -> bool {
    match tok {
        Token::KwReturn | Token::KwEnd | Token::KwElse | Token::KwElseIf | Token::KwUntil => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
