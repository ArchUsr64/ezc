//! Three Address Code Generation
use crate::parser::{self, Program};

/// Tuple struct with `name_index` as the first element and `scope_id` as the next
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BindedIdent(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
	Ident(BindedIdent),
	Temporary(usize),
	Immediate(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RValue {
	Assignment(Operand),
	Operation(Operand, parser::BinaryOperation, Operand),
}

type AddressOffset = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
	Ifz(Operand, AddressOffset),
	Expression(Operand, RValue),
	Return(Operand),
	Goto(isize),
}

/// Assumes the program is semantically sound, should only be ran after
/// `analyzer::analyze` returns `Ok(())`
pub fn generate(program: &Program, ident_count: usize) -> Vec<Instruction> {
	let mut generator = TACGen::new(ident_count);
	generator.scope_gen(&program.0)
}

struct TACGen {
	scope_id: usize,
	scope_map: Vec<Vec<usize>>,
	temp_count: usize,
}
impl TACGen {
	fn new(ident_count: usize) -> Self {
		Self {
			scope_id: 0,
			// TODO: Has rustc automatically pre-allocated required memory or
			// is the vector being resized
			scope_map: (0..ident_count).map(|_| Vec::new()).collect(),
			temp_count: 0,
		}
	}
	fn generate_ident(&self, ident: &parser::Ident) -> BindedIdent {
		let name_index = ident.table_index;
		BindedIdent(name_index, *self.scope_map[name_index].last().unwrap())
	}
	fn generate_rvalue(&self, expr: &parser::Expression) -> RValue {
		use parser::{DirectValue, Expression};
		let to_operand = |direct_value: &DirectValue| -> Operand {
			match direct_value {
				DirectValue::Ident(ident) => Operand::Ident(self.generate_ident(ident)),
				DirectValue::Const(value) => Operand::Immediate(*value),
			}
		};
		match expr {
			Expression::DirectValue(r_value) => RValue::Assignment(to_operand(r_value)),
			Expression::BinaryExpression(l_value, op, r_value) => {
				RValue::Operation(to_operand(l_value), *op, to_operand(r_value))
			}
		}
	}
	fn scope_gen(&mut self, scope: &parser::Scope) -> Vec<Instruction> {
		let mut instructions = Vec::new();
		for stmt in scope.0.iter() {
			use parser::{Ident, Stmts};
			let mut generated_instructions = match stmt {
				Stmts::Decl(Ident { table_index, .. }) => {
					self.scope_map[*table_index].push(self.scope_id);
					Vec::new()
				}
				Stmts::Assignment(ident, expr) => vec![Instruction::Expression(
					Operand::Ident(self.generate_ident(ident)),
					self.generate_rvalue(expr),
				)],
				Stmts::While(expr, scope) => {
					self.scope_id += 1;
					let mut sub_scope = self.scope_gen(scope);
					let mut while_block = vec![
						Instruction::Expression(
							Operand::Temporary(self.temp_count),
							self.generate_rvalue(expr),
						),
						Instruction::Ifz(Operand::Temporary(self.temp_count), sub_scope.len() + 2),
					];
					let scope_len = sub_scope.len();
					while_block.append(&mut sub_scope);
					while_block.push(Instruction::Goto(-(scope_len as isize) - 2));
					while_block
				}
				Stmts::Return(expr) => {
					vec![
						Instruction::Expression(
							Operand::Temporary(self.temp_count),
							self.generate_rvalue(expr),
						),
						Instruction::Return(Operand::Temporary(self.temp_count)),
					]
				}
				Stmts::If(expr, scope) => {
					self.scope_id += 1;
					let mut sub_scope = self.scope_gen(scope);
					let mut if_block = vec![
						Instruction::Expression(
							Operand::Temporary(self.temp_count),
							self.generate_rvalue(expr),
						),
						Instruction::Ifz(Operand::Temporary(self.temp_count), sub_scope.len() + 1),
					];
					if_block.append(&mut sub_scope);
					if_block
				}
				_ => todo!(),
			};
			instructions.append(&mut generated_instructions);
		}
		self.scope_id += 1;
		instructions
	}
}

mod test {
	#[allow(unused_imports)]
	use crate::{
		analyzer::analyze, lexer::tokenize, parser::parse, parser::BinaryOperation, tac_gen,
	};

	#[allow(unused_imports)]
	use super::*;
	#[test]
	fn assignments() {
		let test_program = r"
			int x;
			x = 5;
			return x;
		";
		let tac_expected = vec![
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(5)),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Assignment(Operand::Ident(BindedIdent(0, 0))),
			),
			Instruction::Return(Operand::Temporary(0)),
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}

	#[test]
	fn ifz() {
		let test_program = "if (1) {}";
		let tac_expected = vec![
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Assignment(Operand::Immediate(1)),
			),
			Instruction::Ifz(Operand::Temporary(0), 1),
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));

		let test_program = r"
			int x;
			x = 5;
			if (x <= 4) {
				x = 2;
			}
			return x;
		";
		let tac_expected = vec![
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(5)),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Operation(
					Operand::Ident(BindedIdent(0, 0)),
					BinaryOperation::LessEqual,
					Operand::Immediate(4),
				),
			),
			Instruction::Ifz(Operand::Temporary(0), 2),
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(2)),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Assignment(Operand::Ident(BindedIdent(0, 0))),
			),
			Instruction::Return(Operand::Temporary(0)),
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));

		let test_program = r"
			int x;
			x = 5;
			if (x <= 4) {
				x = x * 2;
				if (x > 9) {
					x = 5;
					x = 9;
					x = 2;
				}
			}
			return x;
		";
		let tac_expected = vec![
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(5)),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Operation(
					Operand::Ident(BindedIdent(0, 0)),
					BinaryOperation::LessEqual,
					Operand::Immediate(4),
				),
			),
			Instruction::Ifz(Operand::Temporary(0), 7),
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Operation(
					Operand::Ident(BindedIdent(0, 0)),
					BinaryOperation::Mul,
					Operand::Immediate(2),
				),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Operation(
					Operand::Ident(BindedIdent(0, 0)),
					BinaryOperation::Greater,
					Operand::Immediate(9),
				),
			),
			Instruction::Ifz(Operand::Temporary(0), 4),
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(5)),
			),
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(9)),
			),
			Instruction::Expression(
				Operand::Ident(BindedIdent(0, 0)),
				RValue::Assignment(Operand::Immediate(2)),
			),
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Assignment(Operand::Ident(BindedIdent(0, 0))),
			),
			Instruction::Return(Operand::Temporary(0)),
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}

	#[test]
	fn while_loops() {
		let test_program = "while (1) {}";
		let tac_expected = vec![
			Instruction::Expression(
				Operand::Temporary(0),
				RValue::Assignment(Operand::Immediate(1)),
			),
			Instruction::Ifz(Operand::Temporary(0), 2),
			Instruction::Goto(-2),
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}
}
