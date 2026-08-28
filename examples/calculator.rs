//! This example showcases a simple calculator application that reads from
//! right to left and can only use single digit operands. Please note that this
//! is not a good way to implement a parser, however it *is* a simple way.

use std::{env, str::CharIndices};

use trogdor::{ErrorHandler, SourceId, Span};

/// We use a wrapper around CharIndices so we can get an index out of `next`,
/// even if we've reached the end of a string.
struct Lexer<'a> {
    pos: u32,
    chars: CharIndices<'a>,
    source_id: SourceId,
}

impl<'a> Lexer<'a> {
    fn new(code: &'a str, source_id: SourceId) -> Self {
        Self {
            pos: 0,
            chars: code.char_indices(),
            source_id,
        }
    }

    fn next(&mut self) -> (Span, Option<char>) {
        if let Some((i, c)) = self.chars.next() {
            let i = i as u32;
            self.pos = i;
            (Span::char(i, self.source_id), Some(c))
        } else {
            (Span::char(self.pos + 1, self.source_id), None)
        }
    }
}

enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

enum Expression {
    Number(f32),
    Operation {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        operation: Operation,
    },
    Error,
}

impl Expression {
    fn parse(lexer: &mut Lexer, eh: &mut ErrorHandler) -> Self {
        let (span, token) = lexer.next();
        match token {
            Some(ch) if let Some(digit) = ch.to_digit(10) => {
                let lhs = Self::Number(digit as f32);
                let (span, token) = lexer.next();
                let operation = match token {
                    Some('+') => Operation::Add,
                    Some('-') => Operation::Sub,
                    Some('*') => Operation::Mul,
                    Some('/') => Operation::Div,
                    Some(ch) => {
                        eh.err(
                            format!("expected the end of input, instead found '{ch}'"),
                            span,
                        );
                        return Self::Error;
                    }
                    None => return lhs,
                };
                let rhs = Box::new(Self::parse(lexer, eh));
                Self::Operation {
                    lhs: Box::new(lhs),
                    rhs,
                    operation,
                }
            }
            Some(ch) => {
                eh.err(
                    format!("expected an expression, instead found '{ch}'"),
                    span,
                );
                Self::Error
            }
            None => {
                eh.err(
                    "expected an expression, instead found the end of input",
                    span,
                );
                Self::Error
            }
        }
    }

    fn calc(&self) -> f32 {
        match self {
            Expression::Number(n) => *n,
            Expression::Operation {
                lhs,
                rhs,
                operation: Operation::Add,
            } => lhs.calc() + rhs.calc(),
            Expression::Operation {
                lhs,
                rhs,
                operation: Operation::Sub,
            } => lhs.calc() - rhs.calc(),
            Expression::Operation {
                lhs,
                rhs,
                operation: Operation::Mul,
            } => lhs.calc() * rhs.calc(),
            Expression::Operation {
                lhs,
                rhs,
                operation: Operation::Div,
            } => lhs.calc() / rhs.calc(),
            Expression::Error => panic!("unhandled error"),
        }
    }
}

pub fn main() {
    let input = env::args().skip(1).next().expect("no program provided");

    let mut eh = ErrorHandler::new();
    let source = eh.add_source("<arg1>", &input);
    let mut lexer = Lexer::new(&input, source);
    let expr = Expression::parse(&mut lexer, &mut eh);

    if eh.is_ok() {
        println!("The answer is {}", expr.calc());
    } else {
        eprintln!("{eh}");
    }
}
