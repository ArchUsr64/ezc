use crate::parser::{DirectValue, Expression, Ident, Program, Scope, Stmts};

#[derive(Debug)]
pub struct SemanticError {
	kind: SemanticErrorKind,
	identifier: Ident,
}
impl SemanticError {
	fn new(kind: SemanticErrorKind, ident: Ident) -> Self {
		Self {
			kind,
			identifier: ident,
		}
	}
}

#[derive(Debug)]
pub enum SemanticErrorKind {
	UseBeforeDeclaration,
	MultipleDeclaration,
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
	let Program {
		global_scope,
		return_value,
		..
	} = program;
	let mut stack = ScopeStack::new();
	stack.scope_analyze(global_scope)?;
	stack
		.expression_valid(return_value)
		.map_err(|ident| SemanticError::new(SemanticErrorKind::UseBeforeDeclaration, ident))
}

type ScopeTable = Vec<usize>;
#[derive(Debug)]
struct ScopeStack(Vec<ScopeTable>);

impl ScopeStack {
	fn new() -> Self {
		Self(Vec::new())
	}
	fn find_ident(&self, ident: &Ident) -> Result<(), Ident> {
		if self
			.0
			.iter()
			.flatten()
			.find(|i| **i == ident.table_index)
			.is_some()
		{
			Ok(())
		} else {
			// TODO: Make ident implement Copy to remove this clone
			Err(ident.clone())
		}
	}
	fn expression_valid(&self, expr: &Expression) -> Result<(), Ident> {
		let find_direct_value = |direct_value: &DirectValue| -> Result<(), Ident> {
			match direct_value {
				DirectValue::Ident(i) => self.find_ident(&i),
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
	fn scope_analyze(&mut self, scope: &Scope) -> Result<(), SemanticError> {
		self.0.push(ScopeTable::new());
		for stmt in scope.statements.iter() {
			use SemanticErrorKind::*;
			match stmt {
				Stmts::Decl(ident) => {
					let current_table = self.0.last_mut().unwrap();
					if current_table.contains(&ident.table_index) {
						return Err(SemanticError::new(MultipleDeclaration, ident.clone()));
					}
					current_table.push(ident.table_index)
				}
				Stmts::Assignment(ident, expr) => {
					if let Err(ident) = self
						.find_ident(&ident)
						.and_then(|_| self.expression_valid(&expr))
					{
						return Err(SemanticError::new(UseBeforeDeclaration, ident));
					}
				}
				Stmts::If(expr, scope) | Stmts::While(expr, scope) => {
					self.expression_valid(&expr)
						.map_err(|ident| SemanticError::new(UseBeforeDeclaration, ident))?;
					self.scope_analyze(&scope)?
				}
			}
		}
		Ok(())
	}
}
