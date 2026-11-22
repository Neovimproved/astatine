use std::str::FromStr;

use derive_more::Display;

use crate::{
    span::Span,
    syntax::{Keyword, Token, TokenKind},
};

#[derive(Debug, Display)]
pub enum TokenizeError {
    #[display("Tried getting the next character but it didn't exist")]
    MissingChar,

    #[display("Multiple periods were found in the number")]
    MultiplePeriods,

    #[display("`{_0}` is not understood by the lexer")]
    UnrecognizedChar(char),

    #[display("Character literal must be only one character long")]
    MultiCharChar(String),

    #[display("You must use a trailing backslash in order to make the string \"{_0}\" multiline")]
    UnescapedMultilineString(String),
}

pub struct Lexer {
    span: Span<char>,
}

impl Lexer {
    pub fn new(s: Vec<char>) -> Self {
        Self { span: Span::new(s) }
    }

    pub fn pos(&self) -> usize {
        self.span.pos()
    }

    fn peek(&mut self) -> Option<char> {
        self.span.peek().copied()
    }

    fn peek_nth(&mut self, idx: usize) -> Option<char> {
        self.span.peek_nth(idx).copied()
    }

    fn next_ok(&mut self) -> Result<char, TokenizeError> {
        self.span.next().copied().ok_or(TokenizeError::MissingChar)
    }

    fn next(&mut self) -> Option<char> {
        self.span.next().copied()
    }

    fn advance_to_next_line(&mut self) {
        while let Some(ch) = self.next()
            && ch != '\n'
        {
            println!("Advancing");
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizeError> {
        let mut tokens = vec![];

        while let Some(ch) = self.peek() {
            let token_kind = match ch {
                'a'..='z' | 'A'..='Z' => self.tokenize_iden_or_keyword()?,

                '0'..='9' => self.tokenize_number()?,

                '"' => self.tokenize_string()?,

                '\'' => self.tokenize_char()?,

                ch => {
                    self.next();

                    match ch {
                        '{' => TokenKind::LeftCurly,

                        '}' => TokenKind::RightCurly,

                        '(' => TokenKind::LeftParen,

                        ')' => TokenKind::RightParen,

                        '[' => TokenKind::LeftSquare,

                        ']' => TokenKind::RightSquare,

                        ':' => TokenKind::Colon,

                        ';' => TokenKind::Semicolon,

                        '+' => TokenKind::Plus,

                        '-' => TokenKind::Minus,

                        '*' => TokenKind::Asterisk,

                        '/' if self.peek_nth(1) == Some('/') => {
                            // indicates a comment
                            self.advance_to_next_line();
                            continue;
                        }

                        '/' => {
                            println!("{:?}", self.peek_nth(1));
                            TokenKind::Slash
                        }

                        ch if ch.is_whitespace() => continue,

                        ch => return Err(TokenizeError::UnrecognizedChar(ch)),
                    }
                }
            };

            tokens.push(Token::new(token_kind, self.span.pos()))
        }

        Ok(tokens)
    }

    fn tokenize_iden_or_keyword(&mut self) -> Result<TokenKind, TokenizeError> {
        let mut acc = String::from(self.next_ok()?);

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() {
                acc.push(ch);
                self.next();
            } else {
                break;
            }
        }

        Ok(match Keyword::from_str(&acc) {
            Ok(keyword) => TokenKind::Keyword(keyword),
            Err(()) => TokenKind::Identifier(acc),
        })
    }

    fn tokenize_number(&mut self) -> Result<TokenKind, TokenizeError> {
        let mut period = false;

        let mut acc = String::from(self.next_ok()?);

        while let Some(ch) = self.peek() {
            match ch {
                '0'..='9' => {
                    acc.push(ch);
                }
                '.' => {
                    if period {
                        return Err(TokenizeError::MultiplePeriods);
                    }

                    period = true;
                    acc.push('.');
                }
                _ => break,
            }

            self.next();
        }

        Ok(if period {
            TokenKind::Integer(acc)
        } else {
            TokenKind::Float(acc)
        })
    }

    fn tokenize_string(&mut self) -> Result<TokenKind, TokenizeError> {
        let mut acc = String::new();
        let mut backslash = false;

        loop {
            let ch = self.next_ok()?;

            if backslash {
                acc.push(ch);
                backslash = false;
            } else {
                match ch {
                    '"' => break,

                    '\n' => return Err(TokenizeError::UnescapedMultilineString(acc)),

                    '\\' => backslash = true,

                    ch => acc.push(ch),
                }
            }
        }

        Ok(TokenKind::StringLiteral(acc))
    }

    fn tokenize_char(&mut self) -> Result<TokenKind, TokenizeError> {
        // remove the leading ' quotation char
        self.next();

        let mut first_ch = self.next_ok()?;
        let mut second_ch = self.next_ok()?;

        if first_ch == '\\' {
            first_ch = second_ch;
            second_ch = self.next_ok()?;
        }

        if second_ch == '\'' {
            Ok(TokenKind::CharLiteral(first_ch))
        } else {
            Err(TokenizeError::MultiCharChar(
                [first_ch, second_ch].iter().collect(),
            ))
        }
    }
}
