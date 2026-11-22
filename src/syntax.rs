use std::{fmt::Display, str::FromStr};

use derive_more::Display;
use pastey::paste;

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

#[derive(Debug, Display)]
pub enum TokenKind {
    /// \*
    Asterisk,

    /// c
    CharLiteral(char),

    /// :
    Colon,

    /// ,
    Comma,

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

macro_rules! impl_try_as {
    (mul; $($kind:ident ($name:ident) $(, $remaining:ident)*);* $(;)?) => {
        $(
            impl_try_as!($kind($name) $(, $remaining)*);
        )*
    };

    // multiple
    ($kind:ident ($first:ident, $($name:ident),*), $ty:expr $(,)?) => {
        paste! {
            impl TokenKind {
                fn [<try_as_ $kind:snake>](&self) -> Result<$ty, TryAsErr> {
                    let Self::$kind($first, $($name),*) = self else {
                        return Err(TryAsErr(stringify!([< $kind:snake >])));
                    };

                    Ok(($first, $($name),*))
                }
            }
        }
    };

    // single with specified type
    ($kind:ident ($name:ident), $ty:expr $(,)?) => {
        paste! {
            impl TokenKind {
                #[allow(dead_code)]
                pub fn [<try_as_ $kind:snake>](&self) -> Result<&$ty, TryAsErr> {
                    let Self::$kind($name) = self else {
                        return Err(TryAsErr(stringify!([< $kind:snake >])));
                    };

                    Ok($name)
                }
            }

            impl Token {
                #[allow(dead_code)]
                pub fn [< try_as_ $kind:snake >](&self) -> Result<&$ty, TryAsErr> {
                    self.kind.[< try_as_ $kind:snake >]()
                }
            }
        }
    };

    // single with default &str type
    ($kind:ident ($name:ident) $(,)?) => {
        paste! {
            impl TokenKind {
                #[allow(dead_code)]
                pub fn [<try_as_ $kind:snake>](&self) -> Result<&str, TryAsErr> {
                    let Self::$kind($name) = self else {
                        return Err(TryAsErr(stringify!([< $kind:snake >])));
                    };

                    Ok($name)
                }
            }

            impl Token {
                #[allow(dead_code)]
                pub fn [< try_as_ $kind:snake> ](&self) -> Result<&str, TryAsErr> {
                    self.kind(). [< try_as_ $kind:snake >]()
                }
            }
        }
    };

    // none variadic
    (none; $($kind:ident);* $(;)?) => {
        $(
        impl_try_as!($kind);
        )*
    };

    // none
    ($kind:ident) => {
        paste! {
            impl TokenKind {
                #[allow(dead_code)]
                pub fn [< try_as_ $kind:snake >] (&self) -> Result<(), TryAsErr> {
                    let Self::$kind = self else {
                        return Err(TryAsErr(stringify!(paste! { [< $kind:snake >] })))
                    };

                    Ok(())
                }
            }

            impl Token {
                #[allow(dead_code)]
                pub fn [< try_as_ $kind:snake >] (&self) -> Result<(), TryAsErr> {
                    self.kind(). [< try_as_ $kind:snake >]()
                }
            }
        }
    }
}

impl_try_as!(none;
    Asterisk;
    Colon;
    LeftCurly;
    LeftParen;
    LeftSquare;
    Minus;
    Plus;
    RightCurly;
    Semicolon;
    Slash;
    Period;
);

impl_try_as!(mul;
    CharLiteral(s), char;
    Float(f);
    Identifier(s);
    StringLiteral(s)
);

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expression {
    Function {
        name: String,
        parameters: Vec<FunctionParameter>,
        statements: Vec<Expression>,
    },
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
