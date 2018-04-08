use lalrpop_util;
use serenity::model::id::UserId;

pub mod ast;

#[allow(unused_imports)]
mod parser;

pub type Error<'a> = lalrpop_util::ParseError<usize, parser::Token<'a>, &'static str>;

pub fn parse_command<'a>(
    cmduser: UserId,
    message: &'a str,
) -> Result<(UserId, ast::Command), Error<'a>> {
    parser::parse_Command(cmduser, message)
}
