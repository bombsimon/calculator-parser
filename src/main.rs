use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io;

#[derive(pest_derive::Parser)]
#[grammar = "calculator.pest"]
pub struct CalculatorParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left) | Op::infix(pow, Left))
            .op(Op::prefix(unary_minus))
    };
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Pow,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Subtract => write!(f, "-"),
            Op::Multiply => write!(f, "×"),
            Op::Divide => write!(f, "÷"),
            Op::Modulo => write!(f, "%"),
            Op::Pow => write!(f, "^"),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    UnaryMinus(Box<Expr>),
    Grouped(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

impl Expr {
    fn eval(&self) -> f64 {
        match self {
            Self::Number(i) => *i,
            Self::UnaryMinus(expr) => -(expr.eval()),
            Self::Grouped(expr) => expr.eval(),
            Self::BinOp { lhs, op, rhs } => {
                let lhs = lhs.eval();
                let rhs = rhs.eval();

                match op {
                    Op::Add => lhs + rhs,
                    Op::Subtract => lhs - rhs,
                    Op::Multiply => lhs * rhs,
                    Op::Divide => lhs / rhs,
                    Op::Modulo => lhs % rhs,
                    Op::Pow => lhs.powf(rhs),
                }
            }
        }
    }
}

impl std::string::ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Self::Number(i) => i.to_string(),
            Self::UnaryMinus(expr) => format!("-{}", expr.to_string()),
            Self::Grouped(expr) => format!("({})", expr.to_string()),
            Self::BinOp { lhs, op, rhs } => {
                format!("{} {} {}", lhs.to_string(), op, rhs.to_string())
            }
        }
    }
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => Expr::Number(primary.as_str().parse::<f64>().unwrap()),
            Rule::expr => Expr::Grouped(Box::new(parse_expr(primary.into_inner()))),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::modulo => Op::Modulo,
                Rule::pow => Op::Pow,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => Expr::UnaryMinus(Box::new(rhs)),
            _ => unreachable!(),
        })
        .parse(pairs)
}

fn main() -> io::Result<()> {
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("› ");
        match readline {
            Ok(line) => {
                if line == "?" {
                    for (i, h) in rl.history().iter().enumerate() {
                        println!("{:<3} {}", i + 1, h);
                        if i >= 9 {
                            break;
                        }
                    }

                    continue;
                }

                rl.add_history_entry(line.as_str()).unwrap();

                match CalculatorParser::parse(Rule::equation, &line) {
                    Ok(mut pairs) => {
                        let expr = parse_expr(pairs.next().unwrap().into_inner());
                        println!("Parsed: {:#?}\n", expr);
                        println!("{} = {}", expr.to_string(), expr.eval());
                    }
                    Err(e) => {
                        eprintln!("Parse failed: {:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    Ok(())
}
