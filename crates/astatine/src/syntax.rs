#![allow(dead_code)]
use std::{fmt::Display, str::FromStr};

use astatine_derive::EnumTryAs;
use derive_more::Display;

use crate::operators::{InfixOperator, PostfixOperator, PrefixOperator};

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

    pub fn into_kind(self) -> TokenKind {
        self.kind
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
pub enum Expression {
    CharLiteral(char),

    FloatLiteral(f64),

    Identifier(String),

    IntegerLiteral(i64),

    StringLiteral(String),

    InfixOp {
        lhs: Box<Expression>,
        op: InfixOperator,
        rhs: Box<Expression>,
    },

    PrefixOp(PrefixOperator, Box<Expression>),

    PostfixOp(Box<Expression>, PostfixOperator),
}

impl Expression {
    // TODO: give it access to symbols or maybe constant fold early
    pub fn try_evaluate(self) -> f64 {
        match self {
            Expression::CharLiteral(_) => todo!(),
            Expression::FloatLiteral(num) => num,
            Expression::Identifier(_) => todo!(),
            Expression::IntegerLiteral(num) => num as f64,
            Expression::StringLiteral(_) => todo!(),
            Expression::InfixOp { lhs, op, rhs } => match op {
                InfixOperator::Add => lhs.try_evaluate() + rhs.try_evaluate(),
                InfixOperator::Sub => lhs.try_evaluate() - rhs.try_evaluate(),
                InfixOperator::Mul => lhs.try_evaluate() * rhs.try_evaluate(),
                InfixOperator::Div => lhs.try_evaluate() / rhs.try_evaluate(),
            },
            Expression::PrefixOp(prefix_operator, expression) => todo!(),
            Expression::PostfixOp(expression, postfix_operator) => todo!(),
        }
    }
}

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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::syntax::Expression;

    macro_rules! express {
        ($expr:expr) => {{
            let s = stringify!($expr);
            ($expr, s)
        }};
    }

    #[test]
    fn expression_evaluation() {
        let (expected, string) = express!(5.0 * (1.0 + 5.0) / 3.0);

        let expr = Expression::from_str(&format!("{string};")).expect("Failed to parse expression");

        assert_eq!(expr.try_evaluate(), expected);
    }
}
