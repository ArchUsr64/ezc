#![feature(let_chains, option_take_if, if_let_guard)]

// mod analyzer;
mod lexer;
mod parser;
// mod tac_gen;
// mod x86_gen;

fn main() {
	env_logger::init();
	let lexer_output = lexer::tokenize(include_str!("test.c"));
	log::debug!("Tokens: {:#?}", lexer_output);
	let (parsed, ident_table) = parser::parse(lexer_output.clone()).unwrap();
	println!("Parse Tree: {parsed:#?}");
	println!("Ident Table: {ident_table:#?}");
	// if let Err(kind) = analyzer::analyze(&parsed) {
	// 	use analyzer::SemanticError;
	// 	match kind {
	// 		SemanticError::UseBeforeDeclaration(ident)
	// 		| SemanticError::MultipleDeclaration(ident) => panic!(
	// 			"Err: '{kind:?}' at '{ident:?}' name: {:?}",
	// 			ident_table.0.get(ident.table_index)
	// 		),
	// 		SemanticError::UndefinedFunction(ident)
	// 		| SemanticError::FunctionRedeclaration(ident) => panic!(
	// 			"Err: '{kind:?}' at '{ident:?}' name: {:?}",
	// 			ident_table.0.get(ident.table_index)
	// 		),
	// 		_ => panic!("Semantic Error: {kind:?}"),
	// 	}
	// }
	// let tac_instructions = tac_gen::generate(&parsed, ident_table.0.len());
	// println!("Code Gen: {tac_instructions:#?}");
	// let x86_asm = x86_gen::x86_gen(tac_instructions, ident_table);
	// log::debug!("x86 Assembly: {x86_asm}");
	// std::fs::write("ezc.asm", x86_asm).unwrap();
}
