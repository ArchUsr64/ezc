//! Grammar:
/*
	<Program>			-> (<Stmts>;)*
	<Stmts>				-> if (<Expression>) {<Stmts>}
						|  while (<Expression>) {<Stmts>};
						|  int Ident;
						|  Ident = <Expression>;
						|  return Expression;
	<Expression>		-> <DirectValue>
						|  <DirectValue> <BinaryOperation> <DirectValue>
	<BinaryOperation>	-> +, -, *, /, &, |, ^, <, <=, >, >=, ==, !=
	<DirectValue>		-> Ident
						|  Const
*/

use std::iter::Peekable;

use crate::lexer::{LexerOutput, Reserved, Symbol, SymbolTable, Token};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOperation {
	Add,
	Sub,
	Mul,
	Div,
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

pub fn parse(lexer_output: LexerOutput) -> Result<(Program, IdentNameTable), Option<Symbol>> {
	let LexerOutput {
		symbol_table: SymbolTable {
			identifier, consts, ..
		},
		symbol,
	} = lexer_output;
	let mut parser = Parser {
		symbols: symbol.iter().map(|i| *i).peekable(),
		const_table: consts,
	};
	let mut statement_root = Vec::new();
	while let Some(stmt) = parser.stmts() {
		statement_root.push(stmt);
	}
	let res = Ok((
		Program {
			global_scope: Scope::new(statement_root),
		},
		IdentNameTable(identifier),
	));
	if parser
		.symbols
		.next_if(|i| {
			matches!(
				i,
				Symbol {
					token: Token::EOF,
					..
				}
			)
		})
		.is_some()
	{
		res
	} else {
		Err(parser.symbols.next())
	}
}

struct Parser<I: Iterator<Item = Symbol>> {
	symbols: Peekable<I>,
	const_table: Vec<String>,
}
impl<I: Iterator<Item = Symbol>> Parser<I> {
	fn next_if_eq(&mut self, needle: Token) -> bool {
		self.symbols.next_if(|&i| i.token == needle).is_some()
	}
	fn next_if(&mut self, func: impl Fn(Token) -> bool) -> Option<Token> {
		self.symbols.next_if(|&i| func(i.token)).map(|i| i.token)
	}
	fn ident(&mut self) -> Option<Ident> {
		match self.symbols.peek() {
			Some(Symbol {
				token: Token::Identifier(index),
				line_number,
			}) => Some(Ident {
				line_number: *line_number,
				table_index: *index,
			})
			.take_if(|_| self.symbols.next().is_some()),
			_ => None,
		}
	}
	fn stmts(&mut self) -> Option<Stmts> {
		if self.next_if_eq(Token::Keyword(Reserved::If)) && self.next_if_eq(Token::LeftParenthesis)
		{
			let expression = self.expression()?;
			if !(self.next_if_eq(Token::RightParenthesis) && self.next_if_eq(Token::LeftBrace)) {
				return None;
			};
			let mut stmts = Vec::new();
			while let Some(stmt) = self.stmts() {
				stmts.push(stmt);
			}
			Some(Stmts::If(expression, Scope::new(stmts)))
				.take_if(|_| self.next_if_eq(Token::RightBrace))
		} else if self.next_if_eq(Token::Keyword(Reserved::While))
			&& self.next_if_eq(Token::LeftParenthesis)
		{
			let expression = self.expression()?;
			if !(self.next_if_eq(Token::RightParenthesis) && self.next_if_eq(Token::LeftBrace)) {
				return None;
			};
			let mut stmts = Vec::new();
			while let Some(stmt) = self.stmts() {
				stmts.push(stmt);
			}
			Some(Stmts::While(expression, Scope::new(stmts)))
				.take_if(|_| self.next_if_eq(Token::RightBrace))
		} else if self.next_if_eq(Token::Keyword(Reserved::Int))
			&& let Some(ident) = self.ident()
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Decl(ident))
		} else if let Some(ident) = self.ident()
			&& self.next_if_eq(Token::Equal)
			&& let Some(expression) = self.expression()
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Assignment(ident, expression))
		} else {
			Some(Stmts::Return(self.return_value()?))
		}
	}
	fn return_value(&mut self) -> Option<Expression> {
		self.next_if_eq(Token::Keyword(Reserved::Return))
			.then_some(())
			.and_then(|_| self.expression())
			.take_if(|_| self.next_if_eq(Token::Semicolon))
	}
	fn expression(&mut self) -> Option<Expression> {
		// TODO: Fix this ugly if else chain
		if let Some(l_value) = self.direct_value() {
			if let Some(binary_operation) = self.binary_operation() {
				if let Some(r_value) = self.direct_value() {
					Some(Expression::BinaryExpression(
						l_value,
						binary_operation,
						r_value,
					))
				} else {
					None
				}
			} else {
				Some(Expression::DirectValue(l_value))
			}
		} else {
			None
		}
	}
	fn direct_value(&mut self) -> Option<DirectValue> {
		if let Some(val) = self.ident() {
			Some(DirectValue::Ident(val))
		} else {
			let sign = if self.next_if_eq(Token::Minus) { -1 } else { 1 };
			match self.next_if(|i| matches!(i, Token::Const(_))) {
				Some(Token::Const(symbol_idx)) => Some(DirectValue::Const(
					sign * self.parse_const(self.const_table.get(symbol_idx)?)?,
				)),
				_ => None,
			}
		}
	}
	fn binary_operation(&mut self) -> Option<BinaryOperation> {
		self.symbols
			.next_if(|tk| BinaryOperation::from_token(&tk.token).is_some())
			.map(|tk| BinaryOperation::from_token(&tk.token))?
	}
	fn parse_const(&self, value: &String) -> Option<i32> {
		if let Ok(val) = i32::from_str_radix(value, 10) {
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

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Program {
	pub global_scope: Scope,
}

#[derive(Clone, Debug)]
pub struct IdentNameTable(pub Vec<String>);

#[derive(Clone, Debug)]
pub struct Scope {
	pub statements: Vec<Stmts>,
}
impl Scope {
	pub fn new(statements: Vec<Stmts>) -> Self {
		Self { statements }
	}
}

/*
   TODO: name here is cloned from the SymbolTable, it should instead just be an
   index into `symbol_table.identifier` and the parser should move it to the
   return value for `fn parse() -> (Program, Vec<String>)`
*/
#[derive(Clone, Debug, PartialEq)]
pub struct Ident {
	line_number: usize,
	pub table_index: usize,
}

#[derive(Clone, Debug)]
pub enum Stmts {
	If(Expression, Scope),
	While(Expression, Scope),
	Decl(Ident),
	Assignment(Ident, Expression),
	Return(Expression),
}

#[derive(Clone, Debug)]
pub enum Expression {
	DirectValue(DirectValue),
	BinaryExpression(DirectValue, BinaryOperation, DirectValue),
}

#[derive(Clone, Debug)]
pub enum DirectValue {
	Ident(Ident),
	Const(i32),
}
