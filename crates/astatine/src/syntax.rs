#![allow(dead_code)]
use std::{fmt::Display, str::FromStr};

use astatine_derive::EnumTryAs;
use derive_more::Display;

#[derive(Debug, Display)]
pub enum Keyword {
    Define,
    Let,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keyword = match s {
            "let" => Self::Let,
            "def" => Self::Define,
            _ => return Err(()),
        };

        Ok(keyword)
    }
}

#[derive(Debug)]
pub struct TryAsErr(&'static str);

impl TryAsErr {
    pub fn tried(&self) -> &'static str {
        self.0
    }
}

impl Display for TryAsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ch) = self.0.chars().next()
            && "aeiou".contains(ch)
        {
            write!(f, "Expected an {}", self.0)
        } else {
            write!(f, "Expected a {}", self.0)
        }
    }
}

#[derive(Debug, Display, EnumTryAs)]
#[error(TryAsErr)]
pub enum TokenKind {
    /// \*
    Asterisk,

    /// c
    CharLiteral(char),

    /// :
    Colon,

    /// ,
    Comma,

    /// =
    Equals,

    /// a.b
    Float(String),

    /// foo
    Identifier(String),

    /// a
    Integer(String),

    /// e.g. def
    Keyword(Keyword),

    /// \{
    LeftCurly,

    /// \(
    LeftParen,

    /// \[
    LeftSquare,

    /// \-
    Minus,

    /// \+
    Plus,

    /// \}
    RightCurly,

    /// \)
    RightParen,

    /// \]
    RightSquare,

    /// ;
    Semicolon,

    /// /
    Slash,

    /// "..."
    StringLiteral(String),

    /// \.
    Period,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    pos: usize,
}

impl Token {
    pub fn new(kind: TokenKind, pos: usize) -> Self {
        Self { kind, pos }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

#[derive(Debug)]
pub enum Node {
    Function {
        name: String,
        parameters: Vec<FunctionParameter>,
        nodes: Vec<FunctionNode>,
    },
}

#[derive(Debug)]
pub enum FunctionNode {
    Declaration {
        name: String,
        value: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum Expression {}

#[derive(Debug)]
pub struct FunctionParameter {
    name: String,
    ty: String,
}

impl FunctionParameter {
    pub fn new(name: String, ty: String) -> Self {
        Self { name, ty }
    }
}
