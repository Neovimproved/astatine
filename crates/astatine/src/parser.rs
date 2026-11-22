#![allow(dead_code)]
use std::ops::{Deref, DerefMut};

use derive_more::{Display, From};

use crate::{
    span::Span,
    syntax::{
        Expression, FunctionNode, FunctionParameter, Keyword, Node, Token, TokenKind, TryAsErr,
    },
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
pub struct Parser {
    tokens: Span<Token>,
}

impl Deref for Parser {
    type Target = Span<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl DerefMut for Parser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tokens
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: Span::new(tokens),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut nodes = vec![];

        while let Some(token) = self.peek() {
            let node = match token.kind() {
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Define => self.parse_function()?,
                    _ => todo!(),
                },
                _ => todo!(),
            };

            nodes.push(node);
        }

        Ok(nodes)
    }

    fn peek_ok(&mut self) -> Result<&Token, ParseError> {
        self.peek().ok_or(ParseError::MissingToken)
    }

    fn next_ok(&mut self) -> Result<&Token, ParseError> {
        self.next().ok_or(ParseError::MissingToken)
    }

    fn parse_function(&mut self) -> Result<Node, ParseError> {
        // advance past definition keyword
        self.next();

        let name = self.next_ok()?.kind().try_as_identifier()?.to_string();

        self.next_ok()?.kind().try_as_left_paren()?;

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

            self.next_ok()?.kind().try_as_colon()?;
            let arg_type = self.next_ok()?.kind().try_as_identifier()?.to_string();

            // TODO: figure out borrow
            parameters.push(FunctionParameter::new(param_name, arg_type));

            if let TokenKind::Comma = self.peek_ok()?.kind() {
                self.next();
            }
        }

        self.next_ok()?.kind().try_as_left_curly()?;

        let mut nodes = vec![];

        // parse nodes
        loop {
            let tok = self.peek_ok()?;

            let node = match tok.kind() {
                TokenKind::Asterisk => todo!(),
                TokenKind::CharLiteral(_) => todo!(),
                TokenKind::Colon => todo!(),
                TokenKind::Comma => todo!(),
                TokenKind::Equals => todo!(),
                TokenKind::Float(_) => todo!(),
                TokenKind::Identifier(_) => todo!(),
                TokenKind::Integer(_) => todo!(),
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Let => self.parse_declaration()?,
                    _ => todo!(),
                },
                TokenKind::LeftCurly => todo!(),
                TokenKind::LeftParen => todo!(),
                TokenKind::LeftSquare => todo!(),
                TokenKind::Minus => todo!(),
                TokenKind::Plus => todo!(),
                TokenKind::RightCurly => break,
                TokenKind::RightParen => todo!(),
                TokenKind::RightSquare => todo!(),
                TokenKind::Semicolon => todo!(),
                TokenKind::Slash => todo!(),
                TokenKind::StringLiteral(_) => todo!(),
                TokenKind::Period => todo!(),
            };

            nodes.push(node);
        }

        Ok(Node::Function {
            name,
            parameters,
            nodes,
        })
    }

    fn parse_function_node(&mut self) -> Result<FunctionNode, ParseError> {
        // TODO: some lookahead shit for detecting hmm is it a function call or is it assignment
        let node = match self.peek_ok()?.kind() {
            TokenKind::Asterisk => todo!(),
            TokenKind::CharLiteral(_) => todo!(),
            TokenKind::Colon => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Equals => todo!(),
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

        Ok(node)
    }

    fn parse_declaration(&mut self) -> Result<FunctionNode, ParseError> {
        // skip declaration keyword
        self.next_ok()?;

        let name = self.next_ok()?.kind().try_as_identifier()?.to_string();

        // skip equals
        self.next_ok()?.kind().try_as_equals()?;

        // pop until semicolon
        let value = self.parse_expression()?;

        Ok(FunctionNode::Declaration {
            name,
            value: Box::new(value),
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        todo!()
    }
}
