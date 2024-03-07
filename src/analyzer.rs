//! Semantic Analyzer
//!
//! Takes a reference to `parser::Program` and returns any errors if present
//! Should be ran before going for code gen, since the later stages expect the
//! program to be semantically sound.
use crate::parser::{DirectValue, Expression, Ident, Program, Scope, Stmts};

#[derive(Debug)]
pub enum SemanticError {
	UseBeforeDeclaration(Ident),
	MultipleDeclaration(Ident),
	ContinueOutsideLoop,
	BreakOutsideLoop,
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
	let Program(functions) = program;
	for function_scope in functions {
		let mut stack = ScopeStack(Vec::new());
		stack.scope_analyze(&function_scope.1, false)?
	}
	Ok(())
}

type ScopeTable = Vec<usize>;
#[derive(Debug)]
struct ScopeStack(Vec<ScopeTable>);

impl ScopeStack {
	fn find_ident(&self, ident: &Ident) -> Result<(), Ident> {
		if self.0.iter().flatten().any(|i| *i == ident.table_index) {
			Ok(())
		} else {
			Err(*ident)
		}
	}
	fn expression_valid(&self, expr: &Expression) -> Result<(), Ident> {
		let find_direct_value = |direct_value: &DirectValue| -> Result<(), Ident> {
			match direct_value {
				DirectValue::Ident(i) => self.find_ident(i),
				DirectValue::Const(_) => Ok(()),
			}
		};
		match expr {
			Expression::DirectValue(d_value) => find_direct_value(d_value),
			Expression::BinaryExpression(l_value, _, r_value) => {
				find_direct_value(l_value).and_then(|_| find_direct_value(r_value))
			}
		}
	}
	fn scope_analyze(&mut self, scope: &Scope, in_loop: bool) -> Result<(), SemanticError> {
		self.0.push(ScopeTable::new());
		for stmt in scope.0.iter() {
			match stmt {
				Stmts::Decl(ident) => {
					let current_table = self.0.last_mut().unwrap();
					if current_table.contains(&ident.table_index) {
						return Err(SemanticError::MultipleDeclaration(*ident));
					}
					current_table.push(ident.table_index)
				}
				Stmts::Assignment(ident, expr) => {
					if let Err(ident) = self
						.find_ident(ident)
						.and_then(|_| self.expression_valid(expr))
					{
						return Err(SemanticError::UseBeforeDeclaration(ident));
					}
				}
				Stmts::If(expr, scope) | Stmts::While(expr, scope) => {
					self.expression_valid(expr)
						.map_err(|ident| SemanticError::UseBeforeDeclaration(ident))?;
					self.scope_analyze(scope, matches!(stmt, Stmts::While(_, _)))?
				}
				Stmts::Return(expr) => {
					if let Err(ident) = self.expression_valid(expr) {
						return Err(SemanticError::UseBeforeDeclaration(ident));
					}
				}
				Stmts::Break => {
					if !in_loop {
						return Err(SemanticError::BreakOutsideLoop);
					}
				}
				Stmts::Continue => {
					if !in_loop {
						return Err(SemanticError::ContinueOutsideLoop);
					}
				}
			}
		}
		self.0.pop();
		Ok(())
	}
}
