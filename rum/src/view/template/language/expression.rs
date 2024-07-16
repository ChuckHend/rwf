use super::{
    super::lexer::{Token, TokenWithContext, Tokenize, Value},
    super::Context,
    super::Error,
    Op, Term,
};

use std::iter::{Iterator, Peekable};

/// An expression, like `5 == 6` or `logged_in == false`,
/// which when evaluated produces a single value, e.g. `true`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Standard `5 + 6`-style expression.
    // It's recusive, so you can have something like `(5 + 6) / (1 - 5)`.
    Binary {
        left: Box<Expression>,
        op: Op,
        right: Box<Expression>,
    },

    Unary {
        op: Op,
        operand: Box<Expression>,
    },

    // Base case for recursive expression parsing, which evaluates to the value
    // of the term, e.g. `5` evalutes to `5` or `variable_name` evalutes to whatever
    // the variable is set to in the context.
    Term {
        term: Term,
    },

    // A list of expressions, e.g.
    // `[1, 2, variable, "hello world"]`
    //
    // The list is dynamically evaluated at runtime, so it can contain variables
    // and constants, as long as the variable is in scope.
    List {
        terms: Vec<Expression>,
    },
}

impl Expression {
    /// Create new constant expression (term).
    pub fn constant(value: Value) -> Self {
        Self::Term {
            term: Term::constant(value),
        }
    }

    /// Create new variable expression (term).
    pub fn variable(variable: String) -> Self {
        Self::Term {
            term: Term::variable(variable),
        }
    }

    /// Evaluate the expression to a value given the context.
    pub fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        match self {
            Expression::Term { term } => term.evaluate(context),
            Expression::Binary { left, op, right } => {
                let left = left.evaluate(context)?;
                let right = right.evaluate(context)?;
                op.evaluate_binary(&left, &right)
            }
            Expression::Unary { op, operand } => {
                let operand = operand.evaluate(context)?;
                op.evaluate_unary(&operand)
            }
            Expression::List { terms } => {
                let mut list = vec![];
                for term in terms {
                    list.push(term.evaluate(context)?);
                }
                Ok(Value::List(list))
            }
        }
    }

    fn term(iter: &mut Peekable<impl Iterator<Item = TokenWithContext>>) -> Result<Self, Error> {
        let next = iter.next().ok_or(Error::Eof)?;
        let term = match next.token() {
            Token::Not => {
                let term = Self::term(iter)?;
                Expression::Unary {
                    op: Op::Not,
                    operand: Box::new(term),
                }
            }
            Token::Minus => {
                let term = Self::term(iter)?;
                Expression::Unary {
                    op: Op::Sub,
                    operand: Box::new(term),
                }
            }
            Token::Plus => {
                let term = Self::term(iter)?;
                Expression::Unary {
                    op: Op::Add,
                    operand: Box::new(term),
                }
            }
            Token::Variable(name) => Self::variable(name),
            Token::Value(value) => Self::constant(value),
            Token::SquareBracketStart => {
                let mut terms = vec![];

                loop {
                    let next = iter.next().ok_or(Error::Eof)?;
                    match next.token() {
                        Token::SquareBracketEnd => break,
                        Token::Comma => continue,
                        Token::Value(value) => terms.push(Expression::constant(value)),
                        Token::Variable(variable) => terms.push(Expression::variable(variable)),
                        _ => return Err(Error::ExpressionSyntax(next)),
                    }
                }

                return Ok(Expression::List { terms });
            }

            Token::RoundBracketStart => {
                let mut count = 1;
                let mut expr = vec![];

                // Count the brackets. The term is finished when the number of opening brackets
                // match the number of closing brackets.
                while count > 0 {
                    let next = iter.peek().ok_or(Error::Eof)?;

                    match next.token() {
                        Token::RoundBracketStart => {
                            count += 1;
                            expr.push(iter.next().ok_or(Error::Eof)?);
                        }
                        Token::RoundBracketEnd => {
                            count -= 1;

                            // If it's not the closing bracket, push it in for recursive parsing later.
                            if count > 0 {
                                expr.push(iter.next().ok_or(Error::Eof)?);
                            } else {
                                // Drop the closing bracket, the expression is over.
                                let _ = iter.next().ok_or(Error::Eof)?;
                            }
                        }
                        Token::BlockEnd => return Err(Error::ExpressionSyntax(next.clone())),

                        _ => {
                            expr.push(iter.next().ok_or(Error::Eof)?);
                        }
                    }
                }

                Self::parse(&mut expr.into_iter().peekable())?
            }

            _ => return Err(Error::ExpressionSyntax(next.clone())),
        };

        Ok(term)
    }

    /// Recusively parse the expression.
    ///
    /// Consumes language tokens automatically.
    pub fn parse(
        iter: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    ) -> Result<Self, Error> {
        // Get the left term, if one exists.
        // TODO: support unary operations.
        let left = Self::term(iter)?;

        // Check if we have another operator.
        let next = iter.peek().ok_or(Error::Eof)?;
        match Op::from_token(next.token()) {
            Some(op) => {
                // We have another operator. Consume the token.
                let _ = iter.next().ok_or(Error::Eof)?;

                // Get the right term. This is a binary op.
                let right = Self::term(iter)?;

                // Check if there's another operator.
                let next = iter.peek();

                match next.map(|t| t.token()) {
                    // Expression is over.
                    Some(Token::BlockEnd) | None => Ok(Expression::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    }),

                    // We have an operator.
                    Some(token) => match Op::from_token(token) {
                        Some(second_op) => {
                            // Consume the token.
                            let _ = iter.next().ok_or(Error::Eof)?;

                            // Get the right term.
                            let right2 = Expression::parse(iter)?;

                            // Check operator precendence.
                            if second_op < op {
                                let expr = Expression::Binary {
                                    left: Box::new(right),
                                    right: Box::new(right2),
                                    op: second_op,
                                };

                                Ok(Expression::Binary {
                                    left: Box::new(left),
                                    right: Box::new(expr),
                                    op,
                                })
                            } else {
                                let left = Expression::Binary {
                                    left: Box::new(left),
                                    right: Box::new(right),
                                    op,
                                };

                                Ok(Expression::Binary {
                                    left: Box::new(left),
                                    right: Box::new(right2),
                                    op: second_op,
                                })
                            }
                        }

                        // Not an op, so syntax error.
                        None => Err(Error::ExpressionSyntax(next.unwrap().clone())),
                    },
                }
            }

            None => return Ok(left),
        }
    }
}

pub trait Evaluate {
    fn evaluate(&self, context: &Context) -> Result<Value, Error>;
    fn evaluate_default(&self) -> Result<Value, Error> {
        self.evaluate(&Context::default())
    }
}

impl Evaluate for &str {
    fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        let tokens = self.tokenize()?[1..].to_vec(); // Skip code block start.
        let expr = Expression::parse(&mut tokens.into_iter().peekable())?;
        expr.evaluate(context)
    }
}

impl Evaluate for String {
    fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        self.as_str().evaluate(context)
    }
}

#[cfg(test)]
mod test {
    use super::super::super::{Context, Tokenize};
    use super::*;

    #[test]
    fn test_if_const() -> Result<(), Error> {
        let t1 = r#"<% 1 == 2 %>"#.tokenize()?;
        let mut iter = t1[1..].to_vec().into_iter().peekable();
        let expr = Expression::parse(&mut iter)?;
        let value = expr.evaluate(&Context::default())?;
        assert_eq!(value, Value::Boolean(false));

        let t2 = "<% 1 && 1 %>".tokenize()?;
        let mut iter = t2[1..].to_vec().into_iter().peekable();
        let expr = Expression::parse(&mut iter)?;
        let value = expr.evaluate(&Context::default())?;
        assert_eq!(value, Value::Boolean(true));

        Ok(())
    }

    #[test]
    fn test_list() -> Result<(), Error> {
        let t1 = r#"<% [1, 2, "hello", 3.13, variable] %>"#.tokenize()?;
        let mut iter = t1[1..].to_vec().into_iter().peekable();
        let ast = Expression::parse(&mut iter)?;

        assert_eq!(iter.next().unwrap().token(), Token::BlockEnd);
        assert!(iter.next().is_none());

        Ok(())
    }

    #[test]
    fn test_op_precendence() -> Result<(), Error> {
        let t1 = r#"<% 2 * 2 + 3 * 5 %>"#.tokenize()?;
        let mut iter = t1[1..].to_vec().into_iter().peekable();
        let ast = Expression::parse(&mut iter)?;
        let context = Context::default();
        let result = ast.evaluate(&context)?;
        assert_eq!(result, Value::Integer(19));
        Ok(())
    }

    #[test]
    fn test_unary() -> Result<(), Error> {
        assert_eq!(
            "<% !false == true && true %>".evaluate_default()?,
            Value::Boolean(true)
        );
        Ok(())
    }

    #[test]
    fn test_parenthesis() -> Result<(), Error> {
        let t1 = "<% ((1 + 2) + (-1 - -1)) * 5 + (25 - 5) %>";
        assert_eq!(t1.evaluate_default()?, Value::Integer(35));

        Ok(())
    }
}
