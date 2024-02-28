//! Three Address Code Generation
use crate::parser::{self, Program};

#[derive(Debug, Clone, Copy)]
pub struct BindedIdent {
	name_index: usize,
	scope_id: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
	Ident(BindedIdent),
	Temporary(usize),
	Immediate(i32),
}

#[derive(Debug, Clone, Copy)]
pub enum RValue {
	Assignment(Operand),
	Operation(Operand, parser::BinaryOperation, Operand),
}

#[derive(Debug, Clone, Copy)]
pub struct AddressOffset(isize);

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
	IFZ(Operand, AddressOffset),
	Expression(Operand, RValue),
	Return(Operand),
}

/// Assumes the program is semantically sound, should only be ran after
/// `analyzer::analyze` returns `Ok(())`
pub fn generate(program: &Program, ident_count: usize) -> Vec<Instruction> {
	let mut generator = TACGen::new(ident_count);
	let mut instructions = generator.scope_gen(&program.global_scope);
	instructions.push(Instruction::Expression(
		Operand::Temporary(generator.temp_count),
		generator.generate_rvalue(&program.return_value),
	));
	instructions.push(Instruction::Return(Operand::Temporary(
		generator.temp_count,
	)));
	instructions
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
		println!("{:?}, {ident:?}", self.scope_map);
		let name_index = ident.table_index;
		BindedIdent {
			name_index,
			scope_id: *self.scope_map[name_index].last().unwrap(),
		}
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
		let current_scope = self.scope_id;
		for stmt in scope.statements.iter() {
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
				Stmts::If(expr, scope) => {
					self.scope_id += 1;
					let mut sub_scope = self.scope_gen(scope);
					let mut if_block = vec![
						Instruction::Expression(
							Operand::Temporary(self.temp_count),
							self.generate_rvalue(expr),
						),
						Instruction::IFZ(
							Operand::Temporary(self.temp_count),
							AddressOffset(sub_scope.len() as isize),
						),
					];
					if_block.append(&mut sub_scope);
					if_block
				}
			};
			instructions.append(&mut generated_instructions);
		}
		// Remove the current scope entires from the scope_map once the scope ends
		// TODO: This global scope check shouldn't be required once return statatments
		// are just regular `Stmts`
		if current_scope != 0 {
			self.scope_map
				.iter_mut()
				.filter(|i| i.last() == Some(&current_scope))
				.for_each(|i| {
					i.pop();
				});
		}
		self.scope_id += 1;
		instructions
	}
}
