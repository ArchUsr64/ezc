//! Three Address Code Generation
use crate::parser::{self, Program};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Ident {
	/// Tuple struct with `name_index` as the first element and `scope_id` as the next
	Binded(usize, usize),
	Parameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
	Ident(Ident),
	Temporary(usize),
	Immediate(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RValue {
	FuncCall(usize),
	Assignment(Operand),
	Operation(Operand, parser::BinaryOperation, Operand),
}

type AddressOffset = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
	Ifz(Operand, AddressOffset),
	Expression(Operand, RValue),
	Return(Operand),
	Push(Operand),
	Goto(isize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
	pub id: usize,
	pub instructions: Vec<Instruction>,
}

/// Assumes the program is semantically sound, should only be ran after
/// `analyzer::analyze` returns `Ok(())`
pub fn generate(program: &Program, ident_count: usize) -> Vec<Function> {
	program
		.0
		.iter()
		.map(|function| {
			let mut generator = TACGen::new(ident_count, function.parameter().table_index);
			Function {
				id: function.name().table_index,
				instructions: generator.generate_scope(function.scope()),
			}
		})
		.collect()
}

struct TACGen {
	scope_id: usize,
	scope_map: Vec<Vec<usize>>,
	parameter: usize,
	temp_count: usize,
}
impl TACGen {
	fn new(ident_count: usize, parameter_index: usize) -> Self {
		Self {
			scope_id: 0,
			// TODO: Has rustc automatically pre-allocated required memory or
			// is the vector being resized
			scope_map: (0..ident_count).map(|_| Vec::new()).collect(),
			parameter: parameter_index,
			temp_count: 0,
		}
	}
	fn generate_ident(&self, ident: &parser::Ident) -> Ident {
		let name_index = ident.table_index;
		if name_index == self.parameter {
			Ident::Parameter
		} else {
			Ident::Binded(name_index, *self.scope_map[name_index].last().unwrap())
		}
	}
	fn generate_assignment(&mut self, lhs: Operand, rhs: &parser::Expression) -> Vec<Instruction> {
		use parser::{DirectValue, Expression};
		let to_operand = |direct_value: &DirectValue| -> Operand {
			match direct_value {
				DirectValue::Ident(ident) => Operand::Ident(self.generate_ident(ident)),
				DirectValue::Const(value) => Operand::Immediate(*value),
			}
		};
		let mut res = Vec::new();
		let r_value = match rhs {
			Expression::FuncCall(func, argument) => {
				res.push(Instruction::Push(to_operand(argument)));
				RValue::FuncCall(func.table_index)
			}
			Expression::DirectValue(r_value) => RValue::Assignment(to_operand(r_value)),
			Expression::BinaryExpression(l_value, op, r_value) => {
				RValue::Operation(to_operand(l_value), *op, to_operand(r_value))
			}
		};
		res.push(Instruction::Expression(lhs, r_value));
		res
	}
	fn generate_scope(&mut self, scope: &parser::Scope) -> Vec<Instruction> {
		let mut instructions = Vec::new();
		for stmt in scope.0.iter() {
			use parser::{Ident, Stmts};
			let mut generated_instructions = match stmt {
				Stmts::Decl(Ident { table_index, .. }) => {
					self.scope_map[*table_index].push(self.scope_id);
					Vec::new()
				}
				Stmts::Assignment(ident, expr) => {
					self.generate_assignment(Operand::Ident(self.generate_ident(ident)), expr)
				}
				Stmts::While(expr, scope) => {
					self.scope_id += 1;
					let mut sub_scope = self.generate_scope(scope);
					let mut while_block =
						self.generate_assignment(Operand::Temporary(self.temp_count), expr);
					while_block.push(Instruction::Ifz(
						Operand::Temporary(self.temp_count),
						sub_scope.len() + 2,
					));
					let scope_len = sub_scope.len();
					while_block.append(&mut sub_scope);
					while_block.push(Instruction::Goto(-(scope_len as isize) - 2));
					while_block
				}
				Stmts::Return(expr) => {
					let mut res =
						self.generate_assignment(Operand::Temporary(self.temp_count), expr);
					res.push(Instruction::Return(Operand::Temporary(self.temp_count)));
					res
				}
				Stmts::If(expr, scope) => {
					self.scope_id += 1;
					let mut sub_scope = self.generate_scope(scope);
					let mut if_block =
						self.generate_assignment(Operand::Temporary(self.temp_count), expr);
					if_block.push(Instruction::Ifz(
						Operand::Temporary(self.temp_count),
						sub_scope.len() + 1,
					));
					if_block.append(&mut sub_scope);
					if_block
				}
				Stmts::Break => todo!(),
				Stmts::Continue => todo!(),
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
			int main(int n) {
				int x;
				x = 5;
				return x;
			}
		";
		let tac_expected = vec![Function {
			id: 0,
			instructions: vec![
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(5)),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Assignment(Operand::Ident(Ident::Binded(2, 0))),
				),
				Instruction::Return(Operand::Temporary(0)),
			],
		}];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}

	#[test]
	fn ifz() {
		let test_program = "int main(int n) {if (1) {}}";
		let tac_expected = vec![Function {
			id: 0,
			instructions: vec![
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Assignment(Operand::Immediate(1)),
				),
				Instruction::Ifz(Operand::Temporary(0), 1),
			],
		}];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));

		let test_program = r"
			int main(int n) {
				int x;
				x = 5;
				if (x <= 4) {
					x = 2;
				}
				return x;
			}
		";
		let tac_expected = vec![Function {
			id: 0,
			instructions: vec![
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(5)),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Operation(
						Operand::Ident(Ident::Binded(2, 0)),
						BinaryOperation::LessEqual,
						Operand::Immediate(4),
					),
				),
				Instruction::Ifz(Operand::Temporary(0), 2),
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(2)),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Assignment(Operand::Ident(Ident::Binded(2, 0))),
				),
				Instruction::Return(Operand::Temporary(0)),
			],
		}];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));

		let test_program = r"
			int main(int n) {
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
			}
			";
		let tac_expected = vec![Function {
			id: 0,
			instructions: vec![
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(5)),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Operation(
						Operand::Ident(Ident::Binded(2, 0)),
						BinaryOperation::LessEqual,
						Operand::Immediate(4),
					),
				),
				Instruction::Ifz(Operand::Temporary(0), 7),
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Operation(
						Operand::Ident(Ident::Binded(2, 0)),
						BinaryOperation::Mul,
						Operand::Immediate(2),
					),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Operation(
						Operand::Ident(Ident::Binded(2, 0)),
						BinaryOperation::Greater,
						Operand::Immediate(9),
					),
				),
				Instruction::Ifz(Operand::Temporary(0), 4),
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(5)),
				),
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(9)),
				),
				Instruction::Expression(
					Operand::Ident(Ident::Binded(2, 0)),
					RValue::Assignment(Operand::Immediate(2)),
				),
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Assignment(Operand::Ident(Ident::Binded(2, 0))),
				),
				Instruction::Return(Operand::Temporary(0)),
			],
		}];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}

	#[test]
	fn while_loops() {
		let test_program = "int main(int n) {while (1) {}}";
		let tac_expected = vec![Function {
			id: 0,
			instructions: vec![
				Instruction::Expression(
					Operand::Temporary(0),
					RValue::Assignment(Operand::Immediate(1)),
				),
				Instruction::Ifz(Operand::Temporary(0), 2),
				Instruction::Goto(-2),
			],
		}];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}

	#[test]
	fn func_calls() {
		let test_program = r"
			int return_1(int n) {
				return 1;
			}
			int main(int n) {
				return return_1(1);
			}
		";
		let tac_expected = vec![
			Function {
				id: 0,
				instructions: vec![
					Instruction::Expression(
						Operand::Temporary(0),
						RValue::Assignment(Operand::Immediate(1)),
					),
					Instruction::Return(Operand::Temporary(0)),
				],
			},
			Function {
				id: 2,
				instructions: vec![
					Instruction::Push(Operand::Immediate(1)),
					Instruction::Expression(Operand::Temporary(0), RValue::FuncCall(0)),
					Instruction::Return(Operand::Temporary(0)),
				],
			},
		];
		let (parsed, table) = parse(tokenize(test_program)).unwrap();
		assert_eq!(tac_expected, generate(&parsed, table.0.len()));
	}
}
