#![feature(let_chains, option_take_if)]

mod lexer;
use lexer::tokenize;
mod parser;
use parser::parse;

const TEST_PROGRAM: &'static str = r"
int x;
x = 10;
if (x == 5) {
	x = x + 5;
	int y;
	y = 10;
	if (y < 20) {
		y = x + y;
	}
}
return x;
";

fn main() {
	println!("Source Code:\n{TEST_PROGRAM}");
	let tokens = tokenize(&TEST_PROGRAM);
	let parsed = parse(tokens.clone());
	println!("Tokens: {:?}", tokens);
	println!("Parse Tree: {:#?}", parsed.unwrap());
}
