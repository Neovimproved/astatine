use std::{collections::HashMap, iter::Peekable, slice::Iter};

use color_eyre::eyre::Result;
use thiserror::Error;

use crate::{
    lexer::Token,
    syntax::{
        Argument, BinaryOp, Declaration, Expression, FunctionDefinition, Keyword, PostfixOp,
        PrefixOp, StructType,
    },
};

pub trait BindingPower {
    fn binding_power(&self) -> u32;
}

impl BindingPower for PrefixOp {
    fn binding_power(&self) -> u32 {
        match self {
            Self::Negate => 101,
            Self::Deref => 101,
            Self::Not => 101,
            Self::BitNot => 101,
            Self::Reference => 101,
        }
    }
}

impl BindingPower for BinaryOp {
    fn binding_power(&self) -> u32 {
        match self {
            Self::Multiply | Self::Divide | Self::Modulo => 100,
            Self::Add | Self::Subtract => 99,
            Self::ShiftL | Self::ShiftR => 98,
            Self::BitAnd => 97,
            Self::Xor => 96,
            Self::BitOr => 95,
            Self::Equal
            | Self::NotEq
            | Self::LessThan
            | Self::GreaterThan
            | Self::LessOrEqual
            | Self::GreaterOrEqual => 94,
            Self::And => 93,
            Self::Or => 92,
        }
    }
}

impl BindingPower for PostfixOp {
    fn binding_power(&self) -> u32 {
        match self {
            Self::Call { args: _ } => 102,
            Self::Index(_) => 102,
            Self::Dot => 102,
            Self::Try => 102,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Expected a token")]
    ExpectedToken,

    #[error("Invalid expression (found `{0:?}`)")]
    InvalidExpression(Expression),

    // TODO: implement display for token, keyword
    #[error("Unexpected token: `{0:?}`; expected {1}")]
    UnexpectedToken(Token, &'static str),

    #[error("Unexpected keyword: `{0:?}`")]
    UnexpectedKeyword(Keyword),
}

trait OptionExt<T> {
    fn or_err(self) -> Result<T, ParseError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_err(self) -> Result<T, ParseError> {
        self.ok_or(ParseError::ExpectedToken)
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Function(FunctionDefinition),

    ConstDecl(Declaration),

    StructDef(StructType),
}

macro_rules! stringify_token {
    ($($variant:tt)*) => {
        stringify!($($variant)*)
            .rsplit("::")
            .next()
            .expect("failed to split token variant")
    };
}

macro_rules! extract_token {
    ($value:expr, $variant:pat) => {
        match $value {
            $variant => {}
            tok => {
                return Err(ParseError::UnexpectedToken(
                    tok.clone(),
                    stringify_token!($variant)
                ))
            }
        }
    };

    ($value:expr, $variant:path, ($($fields:ident),*$(,)?)) => {
        match $value {
            $variant($($fields),*) => ($($fields),*),
            tok => {
                return Err(ParseError::UnexpectedToken(
                    tok.clone(),
                    stringify_token!($variant)
                ))
            }
        }
    };

    ($value:expr, $variant:path, {$($fields:ident),*$(,)?}) => {
        match $value {
            $variant{$($fields),*} => ($($fields),*),
            tok => {
                return Err(ParseError::UnexpectedToken(
                    tok.clone(),
                    stringify_token!($variant)
                ))
            }
        }
    };
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    nodes: Vec<Node>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
            nodes: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Node>, ParseError> {
        while let Some(&token) = self.tokens.peek() {
            let Token::Keyword(keyword) = token else {
                return Err(ParseError::UnexpectedToken(
                    token.clone(),
                    "a keyword at the global scope",
                ));
            };

            let node = match keyword {
                Keyword::Const => self.parse_constant()?,
                Keyword::Func => self.parse_func()?,
                Keyword::Struct => self.parse_struct()?,
                _ => return Err(ParseError::UnexpectedKeyword(keyword.clone())),
            };

            self.nodes.push(node);
        }

        Ok(self.nodes)
    }

    fn parse_declaration(&mut self) -> Result<Expression, ParseError> {
        self.tokens.next().or_err()?;

        let name = extract_token!(self.tokens.next().or_err()?, Token::Identifier, (name));
        extract_token!(self.tokens.next().or_err()?, Token::Eq);

        let expresion = self.parse_expr(0)?;
        extract_token!(self.tokens.next().or_err()?, Token::Semicolon);

        Ok(Expression::Declaration {
            name: name.to_string(),
            value: Box::new(expresion),
        })
    }

    fn parse_return(&mut self) -> Result<Expression, ParseError> {
        self.tokens.next().or_err()?;

        let expression = self.parse_expr(0)?;

        extract_token!(self.tokens.next().or_err()?, Token::Semicolon);

        Ok(Expression::Return(Box::new(expression)))
    }

    // fn parse_infix_op(&mut self, lhs: Expression, op: BinaryOp, rhs) -> Expression {
    //     Expression::InfixOperation {
    //         lhs: Box::new(lhs),
    //         op,
    //         rhs: Box::new(self.tokens.next().unwrap),
    //     }
    // }

    fn parse_expr(&mut self, min_binding_pow: u32) -> Result<Expression, ParseError> {
        let first_token = self.tokens.next().or_err()?;

        let mut lhs = match first_token {
            Token::Literal { kind, value } => Expression::Literal {
                kind: kind.clone(),
                value: value.to_string(),
            },

            Token::Identifier(ident) => Expression::Identifier(ident.to_string()),

            Token::LeftParen => {
                let lhs = self.parse_expr(0)?;
                let token = self.tokens.next().or_err()?;

                if !matches!(token, Token::RightParen) {
                    return Err(ParseError::UnexpectedToken(
                        token.clone(),
                        "a right parenthesis token",
                    ));
                }

                lhs
            }

            // TODO: add prefix operator support
            _ => {
                return Err(ParseError::UnexpectedToken(
                    first_token.clone(),
                    "a valid token for an expression",
                ));
            }
        };

        loop {
            let token = self.tokens.peek().or_err()?;

            let operator = match token {
                Token::RightParen => break,
                Token::LeftSquare => todo!("implement indexing"),
                Token::RightSquare => break,
                Token::Amp => BinaryOp::BitAnd,
                Token::AndAnd => BinaryOp::And,
                Token::Pipe => BinaryOp::BitOr,
                Token::OrOr => BinaryOp::Or,
                Token::BangEq => BinaryOp::NotEq,
                Token::Eq => todo!("maybe implement something for ts later"), // error
                Token::EqEq => BinaryOp::Equal,
                Token::Period => PostfixOp::Dot,
                Token::Comma => todo!(),
                Token::Semicolon => break,
                Token::Asterisk => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,

                // TODO: add postfix operator support
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        (*token).clone(),
                        "infix operator for expression",
                    ));
                }
            };

            let op_bp = operator.binding_power();

            if min_binding_pow >= op_bp {
                break;
            }

            self.tokens.next();
            let rhs = self.parse_expr(op_bp)?;

            lhs = Expression::InfixOperation {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            }
        }

        Ok(lhs)
    }

    fn parse_arg(&mut self) -> Result<Argument, ParseError> {
        // TODO: implement function arguments
        todo!("implement function arguments")
    }

    fn parse_func(&mut self) -> Result<Node, ParseError> {
        self.tokens.next().or_err()?; // discard function marker
        let name_tok = self.tokens.next().or_err()?;

        let Token::Identifier(name) = name_tok else {
            return Err(ParseError::UnexpectedToken(
                name_tok.clone(),
                "an identifier for a function name",
            ));
        };

        // TODO: implement generics (not for a long time though)
        let left_paren_tok = self.tokens.next().or_err()?;

        if !matches!(left_paren_tok, Token::LeftParen) {
            return Err(ParseError::UnexpectedToken(
                left_paren_tok.clone(),
                "a left parenthesis for a function",
            ));
        }

        let mut params = vec![];

        while !matches!(self.tokens.peek().or_err()?, Token::RightParen) {
            params.push(self.parse_arg()?);
        }

        self.tokens.next().or_err()?;

        let mut statements = vec![];

        match self.tokens.next().or_err()? {
            Token::LeftCurly => {}
            tok => return Err(ParseError::UnexpectedToken(tok.clone(), "left curly")),
        }

        while !matches!(self.tokens.peek().or_err()?, Token::RightCurly) {
            statements.push(match self.tokens.peek().or_err()? {
                Token::Keyword(keyword) => match keyword {
                    Keyword::Let => self.parse_declaration()?,
                    Keyword::Return => self.parse_return()?,
                    _ => todo!("implement support for other keywords in statements"),
                },
                _ => todo!("implement other statements"),
            })
        }

        extract_token!(self.tokens.next().or_err()?, Token::RightCurly);

        Ok(Node::Function(FunctionDefinition {
            name: name.to_string(),
            params,
            statements,
        }))
    }

    fn parse_struct(&mut self) -> Result<Node, ParseError> {
        extract_token!(
            self.tokens.next().or_err()?,
            Token::Keyword(Keyword::Struct)
        );

        let struct_name = extract_token!(self.tokens.next().or_err()?, Token::Identifier, (name));

        let mut fields = HashMap::new();

        extract_token!(self.tokens.next().or_err()?, Token::LeftCurly);

        while !matches!(self.tokens.peek(), Some(Token::RightCurly)) {
            let field_name =
                extract_token!(self.tokens.next().or_err()?, Token::Identifier, (name));
            extract_token!(self.tokens.next().or_err()?, Token::Colon);

            let field_type = extract_token!(self.tokens.next().or_err()?, Token::Identifier, (ty));

            fields.insert(field_name.to_string(), field_type.to_string());
            extract_token!(self.tokens.next().or_err()?, Token::Comma);
        }

        self.tokens.next();

        Ok(Node::StructDef(StructType {
            name: struct_name.to_string(),
            fields,
        }))
    }

    fn parse_constant(&mut self) -> Result<Node, ParseError> {
        let expression = self.parse_declaration()?;

        let Expression::Declaration { name, value } = expression else {
            return Err(ParseError::InvalidExpression(expression));
        };

        Ok(Node::ConstDecl(Declaration {
            name,
            value: *value,
        }))
    }
}
