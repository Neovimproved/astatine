#![allow(dead_code)]
use std::{
    num::{ParseFloatError, ParseIntError},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use derive_more::{Display, Error, From};
use miette::IntoDiagnostic;

use crate::{
    lexer::Lexer,
    operators::InfixOperator,
    span::Span,
    syntax::{
        Expression, FunctionNode, FunctionParameter, Keyword, Node, Token, TokenKind, TryAsErr,
    },
};

#[derive(Debug, Display, Error, From)]
pub enum ParseError {
    // FIXME: I know it's not technically grammatically correct but shut up
    #[display("Expected {} {}", if let Some(ch) = _0.chars().next() && "aeiou".contains(ch) { "an" } else { "a" }, _0 )]
    InvalidToken(#[error(not(source))] &'static str),

    #[display("Expected another token but it was missing")]
    MissingToken,

    ParseIntError(ParseIntError),

    ParseFloatError(ParseFloatError),
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
            tokens: Span::new(tokens.into()),
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
                tok => todo!("{self:#?}"),
            };

            nodes.push(node);
        }

        Ok(nodes)
    }

    fn peek_ok(&mut self) -> Result<&Token, ParseError> {
        self.peek().ok_or(ParseError::MissingToken)
    }

    fn next_ok(&mut self) -> Result<Token, ParseError> {
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
        while let Some(node) = self.parse_function_node()? {
            nodes.push(node);
        }

        self.next_ok()?.kind().try_as_right_curly()?;

        Ok(Node::Function {
            name,
            parameters,
            nodes,
        })
    }

    fn parse_function_node(&mut self) -> Result<Option<FunctionNode>, ParseError> {
        // TODO: some lookahead shit for detecting hmm is it a function call or is it assignment
        let node = loop {
            let node = match self.peek_ok()?.kind() {
                TokenKind::Asterisk => todo!(),
                TokenKind::CharLiteral(_) => todo!(),
                TokenKind::Colon => todo!(),
                TokenKind::Comma => todo!(),
                TokenKind::Equals => todo!(),
                TokenKind::Float(_) => todo!(),
                TokenKind::Identifier(_ident) => {
                    todo!("Some lookahead shit");
                }
                TokenKind::Integer(_) => todo!(),
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Define => {
                        return Err(ParseError::InvalidToken(
                            "cannot define function inside function",
                        ));
                    }
                    Keyword::Let => Some(self.parse_declaration()?),
                },
                TokenKind::RightCurly => None,
                TokenKind::LeftCurly => todo!(),
                TokenKind::LeftParen => todo!(),
                TokenKind::LeftSquare => todo!(),
                TokenKind::Minus => todo!(),
                TokenKind::Plus => todo!(),
                TokenKind::RightParen => todo!(),
                TokenKind::RightSquare => todo!(),
                TokenKind::Semicolon => {
                    self.next();
                    continue;
                }
                TokenKind::Slash => todo!(),
                TokenKind::StringLiteral(_) => todo!(),
                TokenKind::Period => todo!(),
            };

            break node;
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
        let value = self.parse_expression(0)?;

        self.next_ok()?.kind().try_as_semicolon()?;

        Ok(FunctionNode::Declaration {
            name,
            value: Box::new(value),
        })
    }

    fn parse_expression(&mut self, min_binding_power: usize) -> Result<Expression, ParseError> {
        let mut lhs = match self.next_ok()?.into_kind() {
            TokenKind::CharLiteral(ch) => Expression::CharLiteral(ch),
            TokenKind::Float(num) => Expression::FloatLiteral(num.parse()?),
            TokenKind::Identifier(ident) => Expression::Identifier(ident),
            TokenKind::Integer(num) => Expression::IntegerLiteral(num.parse()?),
            TokenKind::Keyword(_keyword) => todo!("No keywords are valid here yet"),
            TokenKind::LeftParen => {
                let expr = self.parse_expression(0)?;
                self.next_ok()?.kind().try_as_right_paren()?;
                expr
            }
            TokenKind::RightParen => panic!("Unexpected right paren"),
            TokenKind::Semicolon => panic!("Unexpected semicolon"),
            TokenKind::StringLiteral(lit) => Expression::StringLiteral(lit),
            _ => todo!(),
        };

        loop {
            let op = match self.peek_ok()?.kind() {
                TokenKind::Asterisk => InfixOperator::Mul,
                TokenKind::Minus => InfixOperator::Sub,
                TokenKind::Plus => InfixOperator::Add,
                TokenKind::Slash => InfixOperator::Div,
                TokenKind::Semicolon | TokenKind::RightParen => return Ok(lhs),
                _ => todo!("{self:#?}"),
            };

            println!("Now: {:?}", self.peek());

            let bp = op.binding_power();

            if bp > min_binding_power {
                self.next();

                lhs = Expression::InfixOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(self.parse_expression(bp)?),
                };
            } else {
                return Ok(lhs);
            }
        }
    }
}

impl FromStr for Expression {
    type Err = miette::Error;

    fn from_str(s: &str) -> miette::Result<Self> {
        Self::try_from(s.chars().collect::<Vec<char>>())
    }
}

impl TryFrom<Vec<char>> for Expression {
    type Error = miette::Error;

    fn try_from(value: Vec<char>) -> miette::Result<Self> {
        let mut tokens = Lexer::new(value);
        let mut parser = Parser::new(tokens.tokenize().into_diagnostic()?);

        parser.parse_expression(0).into_diagnostic()
    }
}
