#![feature(let_chains, option_take_if, if_let_guard)]

mod analyzer;
mod lexer;
mod parser;
mod tac_gen;
mod x86_gen;

fn main() {
	let lexer_output = lexer::tokenize(include_str!("test.c"));
	println!("Tokens: {:#?}", lexer_output);
	let (parsed, ident_table) = parser::parse(lexer_output.clone()).unwrap();
	println!("Parse Tree: {parsed:#?}");
	println!("Ident Table: {ident_table:#?}");
	if let Err(analyzer::SemanticError(kind, identifier)) = analyzer::analyze(&parsed) {
		panic!(
			"Err: '{kind:?}' at '{identifier:?}' name: {:?}",
			ident_table.0.get(identifier.table_index)
		)
	}
	let tac_instructions = tac_gen::generate(&parsed, ident_table.0.len());
	println!("Code Gen: {tac_instructions:#?}");
	let x86_asm = x86_gen::x86_gen(tac_instructions);
	println!("x86 Assembly: {x86_asm}");
	std::fs::write("out.asm", x86_asm).unwrap();
}
