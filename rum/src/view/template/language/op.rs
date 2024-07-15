use super::super::lexer::{Token, Value};
use super::super::Error;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Op {
    Not,
    And,
    Or,
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualThan,
    LessThan,
    LessEqualThan,
}

impl Op {
    pub fn from_token(token: Token) -> Option<Self> {
        Option::<Self>::from(token)
    }

    pub fn binary(&self) -> bool {
        match self {
            Op::Not => false,
            _ => true,
        }
    }

    pub fn evaluate_binary(&self, left: &Value, right: &Value) -> Result<Value, Error> {
        match self {
            Op::Equals => Ok(Value::Boolean(left == right)),
            Op::NotEquals => Ok(Value::Boolean(left != right)),
            Op::LessThan => Ok(Value::Boolean(left < right)),
            Op::LessEqualThan => Ok(Value::Boolean(left <= right)),
            Op::GreaterThan => Ok(Value::Boolean(left > right)),
            Op::GreaterEqualThan => Ok(Value::Boolean(left >= right)),
            Op::And => Ok(Value::Boolean(left.truthy() && right.truthy())),
            Op::Or => Ok(Value::Boolean(left.truthy() || right.truthy())),
            _ => todo!(),
        }
    }
}

impl From<Token> for Option<Op> {
    fn from(token: Token) -> Option<Op> {
        Some(match token {
            Token::Not => Op::Not,
            Token::And => Op::And,
            Token::Or => Op::Or,
            Token::Equals => Op::Equals,
            Token::NotEquals => Op::NotEquals,
            Token::GreaterThan => Op::GreaterThan,
            Token::GreaterEqualThan => Op::GreaterEqualThan,
            Token::LessThan => Op::LessThan,
            Token::LessEqualThan => Op::LessEqualThan,
            _ => return None,
        })
    }
}