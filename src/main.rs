#![feature(let_chains, option_take_if, if_let_guard)]

mod lexer;
use lexer::tokenize;

mod parser;
use parser::parse;

mod analyzer;
use analyzer::analyze;

mod tac_gen;

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
	let lexer_output = tokenize(&TEST_PROGRAM);
	// let lexer_output = tokenize(&include_str!("test.c"));
	println!("Tokens: {:#?}", lexer_output);
	let (parsed, ident_table) = parse(lexer_output.clone()).unwrap();
	println!("Parse Tree: {:#?}", parsed);
	println!("Ident Table: {:#?}", ident_table);
	println!("Analysis: {:?}", analyze(&parsed));
	println!(
		"Code Gen: {:#?}",
		tac_gen::generate(&parsed, ident_table.0.len())
	);
}
