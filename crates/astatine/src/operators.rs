#![allow(dead_code)]
use astatine_derive::PoweredEnum;
use derive_more::TryInto;

#[derive(Debug)]
pub enum Operator {
    Infix(InfixOperator),
    Prefix(PrefixOperator),
    Postfix(PostfixOperator),
}

impl Operator {
    pub fn binding_power(&self) -> usize {
        match self {
            Operator::Infix(infix_operator) => infix_operator.binding_power(),
            Operator::Prefix(prefix_operator) => prefix_operator.binding_power(),
            Operator::Postfix(postfix_operator) => postfix_operator.binding_power(),
        }
    }
}

#[derive(Clone, Copy, Debug, PoweredEnum, TryInto)]
pub enum InfixOperator {
    #[power = 9]
    Add,

    #[power = 9]
    Sub,

    #[power = 10]
    Mul,

    #[power = 10]
    Div,
}

#[derive(Debug, PoweredEnum)]
pub enum PrefixOperator {
    #[power = 20]
    BitwiseNot,

    #[power = 20]
    Deref,

    #[power = 20]
    AddressOf,

    #[power = 20]
    LogicalNot,

    #[power = 20]
    Negate,
}

#[derive(Debug, PoweredEnum)]
pub enum PostfixOperator {
    #[power = 21]
    Call,

    #[power = 21]
    Dot,

    #[power = 21]
    Index,
}

#[cfg(test)]
mod test {
    use crate::operators::InfixOperator;

    #[test]
    fn name() {
        let op = InfixOperator::Add;

        assert_eq!(op.binding_power(), 9);
    }
}
