#[macro_use]
extern crate pest_derive;

use std::io::Write;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "calculator.pest"]
pub struct CalculatorParser;

fn main() -> anyhow::Result<()> {
    print!("Enter an expression: ");
    std::io::stdout().flush()?;

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    let mut pairs = CalculatorParser::parse(Rule::expression, &buffer)?;
    parse(pairs.next().ok_or_else(|| anyhow::anyhow!("no pairs"))?);

    Ok(())
}

fn parse(pair: Pair<Rule>) {
    match pair.as_rule() {
        Rule::expression => {
            println!("{:#?}", pair);
        }
        _ => unreachable!(),
    }
}
