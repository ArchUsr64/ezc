//! Semantic Analyzer
//!
//! Takes a reference to `parser::Program` and returns any errors if present
//! Should be ran before going for code gen, since the later stages expect the
//! program to be semantically sound.
use std::collections::HashMap;

use crate::parser::{DirectValue, Expression, FuncSignature, Ident, Program, Scope, Stmts};

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
		let mut stack = ScopeStack::new(func.parameter_table_idx());
		stack.scope_analyze(&func.scope(), ScopeKind::Function, false)?;
		if let Some(err) = stack.function_calls.iter().find_map(|signature| {
			if let Some(parameter_count) = defined_functions.get(&signature.table_index) {
				if *parameter_count != signature.parameter_count {
					Some(SemanticError::InvalidArguments(*signature))
				} else {
					None
				}
			} else {
				Some(SemanticError::UndefinedFunction(*signature))
			}
		}) {
			return Err(err);
		}
	}
	Ok(())
}

type ScopeTable = Vec<usize>;
#[derive(Debug)]
struct ScopeStack {
	scope_table: Vec<ScopeTable>,
	pub function_calls: Vec<FuncSignature>,
}

enum ScopeKind {
	Function,
	Nested,
}

impl ScopeStack {
	fn new(parameters: Vec<usize>) -> Self {
		Self {
			scope_table: vec![parameters],
			function_calls: Vec::new(),
		}
	}
	fn find_ident(&self, ident: &Ident) -> Result<(), Ident> {
		if self
			.scope_table
			.iter()
			.flatten()
			.any(|i| *i == ident.table_index)
		{
			Ok(())
		} else {
			Err(*ident)
		}
	}
	fn expression_valid(&mut self, expr: &Expression) -> Result<(), Ident> {
		let find_direct_value = |direct_value: &DirectValue| -> Result<(), Ident> {
			match direct_value {
				DirectValue::Ident(i) => self.find_ident(i),
				DirectValue::Const(_) => Ok(()),
			}
		};
		match expr {
			Expression::FuncCall(func_name, arguments) => {
				for direct_value in arguments {
					find_direct_value(direct_value)?;
				}
				self.function_calls.push(*func_name);
				Ok(())
			}
			Expression::DirectValue(d_value) => find_direct_value(d_value),
			Expression::BinaryExpression(l_value, _, r_value) => {
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
				Stmts::Decl(ident) => {
					let current_table = self.scope_table.last_mut().unwrap();
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
					self.scope_analyze(
						scope,
						ScopeKind::Nested,
						matches!(stmt, Stmts::While(_, _)) | in_loop,
					)?
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
		self.scope_table.pop();
		Ok(())
	}
}
