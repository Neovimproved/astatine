use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralKind {
    Char,
    Integer,
    Float,
    String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Const,
    Continue,
    Break,
    For,
    Func,
    Else,
    If,
    Let,
    Match,
    Return,
    Struct,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "const" => Ok(Self::Const),
            "continue" => Ok(Self::Continue),
            "break" => Ok(Self::Break),
            "for" => Ok(Self::For),
            "fun" => Ok(Self::Func),
            "else" => Ok(Self::Else),
            "if" => Ok(Self::If),
            "let" => Ok(Self::Let),
            "match" => Ok(Self::Match),
            "return" => Ok(Self::Return),
            "struct" => Ok(Self::Struct),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PrefixOp {
    Negate,
    Deref,
    Not,
    BitNot,
    Reference,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Modulo,

    Add,
    Subtract,

    ShiftL,
    ShiftR,

    BitAnd,
    Xor,
    BitOr,

    Equal,
    NotEq,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,

    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum PostfixOp {
    // Example: print("hello")
    Call { args: Vec<Expression> },

    // Example: list[1]
    Index(Box<Expression>),

    // Example: foo.bar
    Dot,

    // Example: foo()?
    Try,
}

#[derive(Clone, Debug)]
pub enum Expression {
    PrefixOperation {
        op: PrefixOp,
        rhs: Box<Expression>,
    },

    InfixOperation {
        lhs: Box<Expression>,
        op: BinaryOp,
        rhs: Box<Expression>,
    },

    PostfixOperation {
        lhs: Box<Expression>,
        op: PostfixOp,
    },

    Declaration {
        name: String,
        value: Box<Expression>,
    },

    Identifier(String),

    Index {
        lhs: Box<Expression>,
        idx: Box<Expression>,
    },

    Literal {
        kind: LiteralKind,
        value: String,
    },

    Return(Box<Expression>),
}

#[derive(Clone, Debug)]
pub struct Argument {
    value: String,
    value_type: String,
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub params: Vec<Argument>,
    pub statements: Vec<Expression>,
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub enum PrimitiveType {
    Int,
    Float,
    String,
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Self::Primitive(ty) => match ty {
                PrimitiveType::Int => "int",
                PrimitiveType::Float => "float",
                PrimitiveType::String => "string",
            },
            Self::Struct(ty) => ty.name.as_str(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeId(pub usize);

#[derive(Clone, Debug)]
pub struct IdentifierId(pub usize);
