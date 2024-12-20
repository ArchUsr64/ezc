//! Top-Down Recursive Parser
//! Call the `parser::parse` function with lexer's output, returns a `Program`
//!
//! Grammar:
//! ```c
//! <Func>
//! | int Ident(<Parmeter>*) {<Stmts>*}
//!
//! <Parameters>
//! | int Ident
//! | int Ident, <Parameter>
//!
//! <Stmts>
//! | if (<Expression>) {<Stmts>*}
//! | if (<Expression>) <Stmts>
//! | while (<Expression>) {<Stmts>*}
//! | while (<Expression>) <Stmts>
//! | int <Decl>;
//! | Ident [<Expression>] = <Expression>;
//! | Ident = <Expression>;
//! | break;
//! | continue;
//! | return <Expression>;
//!
//! <Decl>
//! | Ident
//! | Ident, <Decl>
//! | Ident [<Const>]
//! | Ident [<Const>], <Decl>
//! | Ident = <Expression>
//! | Ident = <Expression>, <Decl>
//!
//! <Expression>
//! | Ident(<Arguments>)
//! | Ident[<DirectValue>]
//! | <DirectValue>
//! | <DirectValue> <BinaryOperation> <DirectValue>
//!
//! <Arguments>
//! | <DirectValue>
//! | <DirectValue>, <Arguments>
//!
//! <DirectValue>
//! | Ident
//! | Const
//!
//! <BinaryOperation>
//! | +, -, *, /, %, &, |, ^, <, <=, >, >=, ==, !=
//!
//! ```
//! Where a `Program` is just `Vec<Func>`
use std::iter::Peekable;

use crate::lexer::{LexerOutput, Reserved, Symbol, SymbolTable, Token};

/// Returns a parsed `Program` along with an identifier table on successful parse
/// If not, returns the `Symbol` where parsing failed
pub fn parse(lexer_output: LexerOutput) -> Result<(Program, IdentNameTable), Option<Symbol>> {
	let LexerOutput {
		symbol_table: SymbolTable {
			identifier, consts, ..
		},
		symbol,
	} = lexer_output;
	let mut parser = Parser {
		symbols: symbol.iter().copied().peekable(),
		const_table: consts,
	};
	let mut functions = Vec::new();
	while let Some(func) = parser.func() {
		functions.push(func);
	}
	let res = Ok((Program(functions), IdentNameTable(identifier)));
	if parser
		.symbols
		.next_if(|i| matches!(i, Symbol(Token::Eof, ..)))
		.is_some()
	{
		res
	} else {
		Err(parser.symbols.next())
	}
}

#[derive(Clone, Debug)]
pub struct Program(pub Vec<Func>);

#[derive(Clone, Debug)]
pub struct IdentNameTable(pub Vec<String>);

#[derive(Clone, Debug)]
pub struct Scope(pub Vec<Stmts>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ident {
	line_number: usize,
	pub table_index: usize,
}
impl Ident {
	fn as_func_name(&self, parameter_count: usize) -> FuncSignature {
		FuncSignature {
			line_number: self.line_number,
			table_index: self.table_index,
			parameter_count,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuncSignature {
	line_number: usize,
	pub table_index: usize,
	pub parameter_count: usize,
}

/// Tuple struct of the function's name as `Ident` and the respective `Scope`
#[derive(Clone, Debug)]
pub struct Func(FuncSignature, Parameters, Scope);
impl Func {
	fn new(name: Ident, parameters: Parameters, scope: Scope) -> Self {
		Self(
			FuncSignature {
				line_number: name.line_number,
				table_index: name.table_index,
				parameter_count: parameters.len(),
			},
			parameters,
			scope,
		)
	}
	pub fn name(&self) -> FuncSignature {
		self.0
	}
	pub fn parameter(&self) -> &Parameters {
		&self.1
	}
	pub fn parameter_table_idx(&self) -> Vec<usize> {
		self.parameter().iter().map(|i| i.table_index).collect()
	}
	pub fn scope(&self) -> &Scope {
		&self.2
	}
}

pub type Parameters = Vec<Ident>;

#[derive(Clone, Debug)]
pub enum Decl {
	Array {
		name: Ident,
		size: u32,
	},
	Variable {
		name: Ident,
		init_val: Option<Expression>,
	},
}

#[derive(Clone, Debug)]
pub enum Stmts {
	If(Expression, Scope),
	While(Expression, Scope),
	Decl(Vec<Decl>),
	Assignment(Ident, Expression),
	ArrayAssignment(Ident, Expression, Expression),
	Break,
	Continue,
	Return(Expression),
}

#[derive(Clone, Debug)]
pub enum Expression {
	FuncCall(FuncSignature, Arguments),
	ArrayAccess(Ident, DirectValue),
	DirectValue(DirectValue),
	Binary(DirectValue, BinaryOperation, DirectValue),
}

type Arguments = Vec<DirectValue>;

#[derive(Clone, Debug)]
pub enum DirectValue {
	Ident(Ident),
	Const(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperation {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	And,
	Or,
	Xor,
	Less,
	LessEqual,
	Greater,
	GreaterEqual,
	Equal,
	NotEqual,
}
impl BinaryOperation {
	fn from_token(token: &Token) -> Option<BinaryOperation> {
		use Token::*;
		match token {
			Plus => Some(Self::Add),
			Minus => Some(Self::Sub),
			Star => Some(Self::Mul),
			Slash => Some(Self::Div),
			Percent => Some(Self::Mod),
			Amp => Some(Self::And),
			Pipe => Some(Self::Or),
			Caret => Some(Self::Xor),
			Less => Some(Self::Less),
			LessEqual => Some(Self::LessEqual),
			Greater => Some(Self::Greater),
			GreaterEqual => Some(Self::GreaterEqual),
			EqualEqual => Some(Self::Equal),
			BangEqual => Some(Self::NotEqual),
			_ => None,
		}
	}
}

/// Manages the state of the input `Symbol` stream during parsing
#[derive(Debug)]
struct Parser<I: Iterator<Item = Symbol> + std::fmt::Debug> {
	symbols: Peekable<I>,
	const_table: Vec<String>,
}
impl<I: Iterator<Item = Symbol> + std::fmt::Debug> Parser<I> {
	fn peek(&mut self) -> Option<Symbol> {
		self.symbols.peek().copied()
	}
	fn tk_peek(&mut self) -> Option<Token> {
		self.peek().map(|s| s.0)
	}
	fn next_if_eq(&mut self, needle: Token) -> bool {
		self.symbols.next_if(|&i| i.token() == needle).is_some()
	}
	fn next_if(&mut self, func: impl Fn(Token) -> bool) -> Option<Token> {
		self.symbols
			.next_if(|&i| func(i.token()))
			.map(|i| i.token())
	}
	fn ident(&mut self) -> Option<Ident> {
		match self.peek() {
			Some(Symbol(Token::Identifier(index), line_number)) => Some(Ident {
				line_number,
				table_index: index,
			})
			.take_if(|_| self.symbols.next().is_some()),
			_ => None,
		}
	}
	fn func(&mut self) -> Option<Func> {
		let mut scope = Vec::new();
		if self.next_if_eq(Token::Keyword(Reserved::Int))
			&& let Some(id) = self.ident()
			&& self.next_if_eq(Token::LeftParenthesis)
			&& let Some(parameter) = self.parameters()
			&& self.next_if_eq(Token::RightParenthesis)
			&& self.next_if_eq(Token::LeftBrace)
		{
			while let Some(stmt) = self.stmts() {
				scope.push(stmt);
			}
			if self.next_if_eq(Token::RightBrace) {
				Some(Func::new(id, parameter, Scope(scope)))
			} else {
				None
			}
		} else {
			None
		}
	}
	fn decl(&mut self) -> Option<Vec<Decl>> {
		let mut res = Vec::new();
		while !matches!(self.tk_peek(), Some(Token::Semicolon)) {
			if !res.is_empty() && !self.next_if_eq(Token::Comma) {
				return None;
			}
			if let Some(name) = self.ident() {
				if self.next_if_eq(Token::Equal) {
					res.push(Decl::Variable {
						name,
						init_val: self.expression(),
					});
				} else if self.next_if_eq(Token::LeftSquare) {
					res.push(Decl::Array {
						name,
						size: self.constant()? as u32,
					});
					if !self.next_if_eq(Token::RightSquare) {
						return None;
					}
				} else {
					res.push(Decl::Variable {
						name,
						init_val: None,
					});
				}
			} else {
				return None;
			}
		}
		Some(res)
	}
	fn parameters(&mut self) -> Option<Parameters> {
		let mut res = Vec::new();
		while !matches!(self.tk_peek(), Some(Token::RightParenthesis)) {
			if !res.is_empty() && !self.next_if_eq(Token::Comma) {
				return None;
			}
			if self.next_if_eq(Token::Keyword(Reserved::Int))
				&& let Some(ident) = self.ident()
			{
				res.push(ident);
			} else {
				return None;
			}
		}
		Some(res)
	}
	fn arguments(&mut self) -> Option<Arguments> {
		let mut res = Vec::new();
		while !matches!(self.tk_peek(), Some(Token::RightParenthesis)) {
			if !res.is_empty() && !self.next_if_eq(Token::Comma) {
				return None;
			}
			if let Some(direct_val) = self.direct_value() {
				res.push(direct_val);
			} else {
				return None;
			}
		}
		Some(res)
	}
	fn stmts(&mut self) -> Option<Stmts> {
		if self.next_if_eq(Token::Keyword(Reserved::If)) && self.next_if_eq(Token::LeftParenthesis)
		{
			let expression = self.expression()?;
			if !self.next_if_eq(Token::RightParenthesis) {
				return None;
			};
			Some(Stmts::If(expression, Scope(self.stmts_body()?)))
		} else if self.next_if_eq(Token::Keyword(Reserved::While))
			&& self.next_if_eq(Token::LeftParenthesis)
		{
			let expression = self.expression()?;
			if !self.next_if_eq(Token::RightParenthesis) {
				return None;
			};
			Some(Stmts::While(expression, Scope(self.stmts_body()?)))
		} else if self.next_if_eq(Token::Keyword(Reserved::Int))
			&& let Some(decl) = self.decl()
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Decl(decl))
		} else if let Some(ident) = self.ident() {
			if self.next_if_eq(Token::Equal)
				&& let Some(expression) = self.expression()
				&& self.next_if_eq(Token::Semicolon)
			{
				Some(Stmts::Assignment(ident, expression))
			} else if self.next_if_eq(Token::LeftSquare)
				&& let Some(index) = self.expression()
				&& self.next_if_eq(Token::RightSquare)
				&& self.next_if_eq(Token::Equal)
				&& let Some(r_value) = self.expression()
				&& self.next_if_eq(Token::Semicolon)
			{
				Some(Stmts::ArrayAssignment(ident, index, r_value))
			} else {
				None
			}
		} else if self.next_if_eq(Token::Keyword(Reserved::Break))
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Break)
		} else if self.next_if_eq(Token::Keyword(Reserved::Continue))
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Continue)
		} else {
			Some(Stmts::Return(
				self.next_if_eq(Token::Keyword(Reserved::Return))
					.then_some(())
					.and_then(|_| self.expression())
					.take_if(|_| self.next_if_eq(Token::Semicolon))?,
			))
		}
	}
	/// Equivalent to either:
	/// { <Stmts>* } OR <Stmts>
	/// Used for parsing the body for if and while
	fn stmts_body(&mut self) -> Option<Vec<Stmts>> {
		if self.next_if_eq(Token::LeftBrace) {
			let mut stmts = Vec::new();
			while let Some(stmt) = self.stmts() {
				stmts.push(stmt);
			}
			Some(stmts).take_if(|_| self.next_if_eq(Token::RightBrace))
		} else {
			Some(vec![self.stmts()?])
		}
	}
	fn expression(&mut self) -> Option<Expression> {
		let l_value = self.direct_value()?;
		if let DirectValue::Ident(ident) = l_value {
			if self.next_if_eq(Token::LeftParenthesis) {
				if let Some(arguments) = self.arguments()
					&& self.next_if_eq(Token::RightParenthesis)
				{
					return Some(Expression::FuncCall(
						ident.as_func_name(arguments.len()),
						arguments,
					));
				} else {
					return None;
				}
			}
			if self.next_if_eq(Token::LeftSquare) {
				if let Some(size) = self.direct_value()
					&& self.next_if_eq(Token::RightSquare)
				{
					return Some(Expression::ArrayAccess(ident, size));
				} else {
					return None;
				}
			}
		}
		if let Some(binary_operation) = self.binary_operation() {
			Some(Expression::Binary(
				l_value,
				binary_operation,
				self.direct_value()?,
			))
		} else {
			Some(Expression::DirectValue(l_value))
		}
	}
	fn direct_value(&mut self) -> Option<DirectValue> {
		if let Some(val) = self.ident() {
			Some(DirectValue::Ident(val))
		} else {
			self.constant().map(DirectValue::Const)
		}
	}
	fn constant(&mut self) -> Option<i32> {
		let sign = if self.next_if_eq(Token::Minus) { -1 } else { 1 };
		match self.next_if(|i| matches!(i, Token::Const(_))) {
			Some(Token::Const(symbol_idx)) => {
				Some(sign * self.parse_const(self.const_table.get(symbol_idx)?)?)
			}
			_ => None,
		}
	}
	fn binary_operation(&mut self) -> Option<BinaryOperation> {
		self.next_if(|tk| BinaryOperation::from_token(&tk).is_some())
			.map(|tk| BinaryOperation::from_token(&tk))?
	}
	fn parse_const(&self, value: &str) -> Option<i32> {
		if let Ok(val) = value.parse::<i32>() {
			Some(val)
		} else {
			let value = value.trim_start_matches('0');
			let radix = match value.chars().nth(0)? {
				'b' => Some(2),
				'o' => Some(8),
				'x' => Some(16),
				_ => None,
			};
			i32::from_str_radix(&value[1..], radix?).ok()
		}
	}
}
