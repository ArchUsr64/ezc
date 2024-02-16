#![feature(let_chains, option_take_if)]

mod lexer;
use lexer::tokenize;
mod parser;
use parser::parse;

const TEST_PROGRAM: &'static str = r"
int x;
int y;
int z;
z = y;
z = 10;
if (x + 9) {
	y = 10 + z;
}
return x;
";

fn main() {
	println!("{TEST_PROGRAM}");
	let tokens = tokenize(&TEST_PROGRAM);
	let parsed = parse(tokens.clone());
	println!("{:?}", tokens);
	println!("{:#?}", parsed.unwrap());
}
