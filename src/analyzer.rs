//! Semantic Analyzer
//!
//! Takes a reference to `parser::Program` and returns any errors if present
//! Should be ran before going for code gen, since the later stages expect the
//! program to be semantically sound.
use std::collections::HashMap;

use crate::parser::{Decl, DirectValue, Expression, FuncSignature, Ident, Program, Scope, Stmts};

#[derive(Debug)]
pub enum SemanticError {
	UndefinedFunction(FuncSignature),
	FunctionRedeclaration(FuncSignature),
	UseBeforeDeclaration(Ident),
	MultipleDeclaration(Ident),
	ContinueOutsideLoop,
	BreakOutsideLoop,
	InvalidArguments(FuncSignature),
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
	let Program(functions) = program;
	let mut defined_functions = HashMap::new();
	for func in functions {
		if let Some(_prev_decl) =
			defined_functions.insert(func.name().table_index, func.parameter().len())
		{
			return Err(SemanticError::FunctionRedeclaration(func.name()));
		}
		let mut stack = ScopeStack::new(func.parameter_table_idx(), &defined_functions);
		stack.scope_analyze(func.scope(), ScopeKind::Function, false)?;
	}
	Ok(())
}

type ScopeTable = Vec<usize>;
#[derive(Debug)]
struct ScopeStack<'a> {
	scope_table: Vec<ScopeTable>,
	defined_functions: &'a HashMap<usize, usize>,
}

enum ScopeKind {
	Function,
	Nested,
}

impl<'a> ScopeStack<'a> {
	fn new(parameters: Vec<usize>, defined_functions: &'a HashMap<usize, usize>) -> Self {
		Self {
			scope_table: vec![parameters],
			defined_functions,
		}
	}
	fn find_ident(&self, ident: &Ident) -> bool {
		self.scope_table
			.iter()
			.flatten()
			.any(|i| *i == ident.table_index)
	}
	fn expression_valid(&mut self, expr: &Expression) -> Result<(), SemanticError> {
		let find_direct_value = |direct_value: &DirectValue| -> Result<(), SemanticError> {
			match direct_value {
				DirectValue::Ident(i) => {
					if self.find_ident(i) {
						Ok(())
					} else {
						Err(SemanticError::UseBeforeDeclaration(*i))
					}
				}
				DirectValue::Const(_) => Ok(()),
			}
		};
		match expr {
			Expression::FuncCall(sig, arguments) => {
				let arg_count = self.defined_functions.get(&sig.table_index).copied();
				if arg_count.is_none() {
					return Err(SemanticError::UndefinedFunction(*sig));
				}
				if arg_count != Some(arguments.len()) {
					return Err(SemanticError::InvalidArguments(*sig));
				}
				for direct_value in arguments {
					find_direct_value(direct_value)?;
				}
				Ok(())
			}
			Expression::DirectValue(d_value) => find_direct_value(d_value),
			Expression::Binary(l_value, _, r_value) => {
				find_direct_value(l_value).and_then(|_| find_direct_value(r_value))
			}
		}
	}
	fn scope_analyze(
		&mut self,
		scope: &Scope,
		scope_kind: ScopeKind,
		in_loop: bool,
	) -> Result<(), SemanticError> {
		if let ScopeKind::Nested = scope_kind {
			self.scope_table.push(ScopeTable::new());
		}
		for stmt in scope.0.iter() {
			match stmt {
				Stmts::Decl(decls) => {
					for Decl(ident, expr) in decls {
						if self
							.scope_table
							.last()
							.unwrap()
							.contains(&ident.table_index)
						{
							return Err(SemanticError::MultipleDeclaration(*ident));
						}
						if let Some(expr) = expr {
							self.expression_valid(expr)?;
						}
						self.scope_table.last_mut().unwrap().push(ident.table_index)
					}
				}
				Stmts::Assignment(ident, expr) => {
					if self.find_ident(ident) {
						self.expression_valid(expr)?
					} else {
						return Err(SemanticError::UseBeforeDeclaration(*ident));
					}
				}
				Stmts::If(expr, scope) | Stmts::While(expr, scope) => {
					self.expression_valid(expr)?;
					self.scope_analyze(
						scope,
						ScopeKind::Nested,
						matches!(stmt, Stmts::While(_, _)) | in_loop,
					)?
				}
				Stmts::Return(expr) => self.expression_valid(expr)?,
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
		self.scope_table.pop();
		Ok(())
	}
}
