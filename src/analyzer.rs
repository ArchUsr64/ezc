use crate::parser::{DirectValue, Expression, Ident, Program, Scope, Stmts};

#[derive(Debug)]
pub enum SemanticError {
	UseBeforeDeclaration,
	MultipleDeclaration,
}

pub fn analyze(program: Program) -> Result<(), SemanticError> {
	let Program {
		global_scope,
		return_value,
	} = program;
	let mut stack = ScopeStack::new();
	stack.scope_analyze(global_scope)?;
	if stack.expression_valid(return_value) {
		Ok(())
	} else {
		Err(SemanticError::UseBeforeDeclaration)
	}
}

type ScopeTable = Vec<String>;
#[derive(Debug)]
struct ScopeStack(Vec<ScopeTable>);

impl ScopeStack {
	fn new() -> Self {
		Self(Vec::new())
	}
	fn find_ident(&self, ident: &Ident) -> bool {
		self.0
			.iter()
			.flatten()
			.find(|i| **i == ident.name)
			.is_some()
	}
	fn expression_valid(&self, expr: Expression) -> bool {
		let find_direct_value = |direct_value| -> bool {
			match direct_value {
				DirectValue::Ident(i) => self.find_ident(&i),
				DirectValue::Const(_) => true,
			}
		};
		match expr {
			Expression::DirectValue(d_value) => find_direct_value(d_value),
			Expression::BinaryExpression(l_value, _, r_value) => {
				find_direct_value(l_value) && find_direct_value(r_value)
			}
		}
	}
	fn scope_analyze(&mut self, scope: Scope) -> Result<(), SemanticError> {
		self.0.push(ScopeTable::new());
		for stmt in scope.statements {
			match stmt {
				Stmts::Decl(Ident { name, .. }) => {
					let current_table = self.0.last_mut().unwrap();
					if current_table.contains(&name) {
						return Err(SemanticError::MultipleDeclaration);
					}
					current_table.push(name)
				}
				Stmts::Assignment(ident, expr) => {
					if !(self.find_ident(&ident) && self.expression_valid(expr)) {
						return Err(SemanticError::UseBeforeDeclaration);
					}
				}
				Stmts::If(expr, scope) => {
					if !self.expression_valid(expr) {
						return Err(SemanticError::UseBeforeDeclaration);
					};
					self.scope_analyze(scope)?
				}
			}
		}
		Ok(())
	}
}
