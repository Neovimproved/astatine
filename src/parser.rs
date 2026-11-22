#![allow(dead_code)]
use std::{
    ops::{Deref, DerefMut},
    slice::Iter,
};

use derive_more::{Display, From};

use crate::{
    span::Span,
    syntax::{Expression, FunctionParameter, Keyword, Token, TokenKind, TryAsErr},
};

#[derive(Debug, Display, From)]
pub enum ParseError {
    // FIXME: I know it's not technically grammatically correct but shut up
    #[display("Expected {} {}", if let Some(ch) = _0.chars().next() && "aeiou".contains(ch) { "an" } else { "a" }, _0 )]
    InvalidToken(&'static str),

    #[display("Expected another token but it was missing")]
    MissingToken,
}

impl From<TryAsErr> for ParseError {
    fn from(value: TryAsErr) -> Self {
        Self::InvalidToken(value.tried())
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Span<Iter<'a, Token>>,
}

impl<'a> Deref for Parser<'a> {
    type Target = Span<Iter<'a, Token>>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl<'a> DerefMut for Parser<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tokens
    }
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: Span::new(tokens.iter()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut expressions = vec![];

        while let Some(token) = self.peek() {
            let expr = match token.kind() {
                TokenKind::Asterisk => todo!(),
                TokenKind::CharLiteral(_) => todo!(),
                TokenKind::Colon => todo!(),
                TokenKind::Comma => todo!(),
                TokenKind::Float(_) => todo!(),
                TokenKind::Identifier(_) => todo!(),
                TokenKind::Integer(_) => todo!(),
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Define => self.parse_function()?,
                    _ => todo!(),
                },
                TokenKind::LeftCurly => todo!(),
                TokenKind::LeftParen => todo!(),
                TokenKind::LeftSquare => todo!(),
                TokenKind::Minus => todo!(),
                TokenKind::Plus => todo!(),
                TokenKind::RightCurly => todo!(),
                TokenKind::RightParen => todo!(),
                TokenKind::RightSquare => todo!(),
                TokenKind::Semicolon => todo!(),
                TokenKind::Slash => todo!(),
                TokenKind::StringLiteral(_) => todo!(),
                TokenKind::Period => todo!(),
            };

            expressions.push(expr);
        }

        Ok(expressions)
    }

    fn peek_ok(&mut self) -> Result<&Token, ParseError> {
        self.peek().ok_or(ParseError::MissingToken)
    }

    fn next_ok(&mut self) -> Result<&Token, ParseError> {
        self.next().ok_or(ParseError::MissingToken)
    }

    fn parse_function(&mut self) -> Result<Expression, ParseError> {
        // advance past definition keyword
        self.next();

        let name = self.next_ok()?.try_as_identifier()?.to_string();

        self.next_ok()?.try_as_left_paren()?;

        let mut parameters = vec![];

        // parse args
        loop {
            let tok = self.next_ok()?;

            let param_name = match tok.kind() {
                TokenKind::Identifier(ident) => ident.to_string(),
                TokenKind::RightParen => break,
                _ => {
                    return Err(ParseError::InvalidToken(
                        "right parentheis or next parameter",
                    ));
                }
            };

            self.next_ok()?.try_as_colon()?;
            let arg_type = self.next_ok()?.try_as_identifier()?.to_string();

            // TODO: figure out borrow
            parameters.push(FunctionParameter::new(param_name, arg_type));

            if let TokenKind::Comma = self.peek_ok()?.kind() {
                self.next();
            }
        }

        self.next_ok()?.try_as_left_curly()?;

        loop {
            let tok = self.next_ok()?;

            if matches!(tok.kind(), TokenKind::RightCurly) {
                break;
            }
        }

        Ok(Expression::Function {
            name,
            parameters,
            statements: vec![],
        })
    }

    fn parse_statement(&mut self) -> Result<Expression, ParseError> {
        // TODO: some lookahead shit for detecting hmm is it a function call or is it assignment
        let expr = match self.peek_ok()?.kind() {
            TokenKind::Asterisk => todo!(),
            TokenKind::CharLiteral(_) => todo!(),
            TokenKind::Colon => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Float(_) => todo!(),
            TokenKind::Identifier(_) => todo!(),
            TokenKind::Integer(_) => todo!(),
            TokenKind::Keyword(keyword) => match keyword {
                Keyword::Define => {
                    return Err(ParseError::InvalidToken("statement in function definition"));
                }
                Keyword::Let => self.parse_declaration()?,
            },
            TokenKind::LeftCurly => todo!(),
            TokenKind::LeftParen => todo!(),
            TokenKind::LeftSquare => todo!(),
            TokenKind::Minus => todo!(),
            TokenKind::Plus => todo!(),
            TokenKind::RightCurly => todo!(),
            TokenKind::RightParen => todo!(),
            TokenKind::RightSquare => todo!(),
            TokenKind::Semicolon => todo!(),
            TokenKind::Slash => todo!(),
            TokenKind::StringLiteral(_) => todo!(),
            TokenKind::Period => todo!(),
        };

        Ok(expr)
    }

    fn parse_declaration(&mut self) -> Result<Expression, ParseError> {
        self.tokens.
    }
}
