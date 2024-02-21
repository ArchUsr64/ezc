//! Grammar:
/*
	<Program>			-> <Stmts> return <Expression>;
	<Stmts>				-> if (<Expression>) {<Stmts>}
						|  int Ident;
						|  Ident = <Expression>;
	<Expression>		-> <DirectValue>
						|  <DirectValue> <BinaryOperation> <DirectValue>
	<BinaryOperation>	-> +, -, *, /, &, |, ^, <, <=, >, >=, ==, !=
	<DirectValue>		-> Ident
						|  Const
*/

use std::iter::Peekable;

use crate::lexer::{LexerOutput, Reserved, Symbol, SymbolTable, Token};

#[derive(Clone, Debug, PartialEq)]
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

pub fn parse(lexer_output: LexerOutput) -> Result<Program, Option<Symbol>> {
	let LexerOutput {
		symbol_table,
		symbol,
	} = lexer_output;
	let mut parser = Parser {
		symbols: symbol.iter().map(|i| *i).peekable(),
		symbol_table,
	};
	let mut statement_root = Vec::new();
	while let Some(stmt) = parser.stmts() {
		statement_root.push(stmt);
	}
	let res = Ok(Program {
		statement_root,
		return_value: parser
			.return_value()
			.ok_or_else(|| parser.symbols.peek().map(|i| *i))?,
	});
	if let Some(Symbol {
		token: Token::EOF, ..
	}) = parser.symbols.peek()
	{
		res
	} else {
		Err(parser.symbols.next())
	}
}

struct Parser<I: Iterator<Item = Symbol>> {
	symbols: Peekable<I>,
	symbol_table: SymbolTable,
}
impl<I: Iterator<Item = Symbol>> Parser<I> {
	#[inline]
	fn next_if_eq(&mut self, needle: Token) -> bool {
		self.symbols.next_if(|&i| i.token == needle).is_some()
	}
	fn next_if(&mut self, func: impl Fn(Token) -> bool) -> Option<Token> {
		self.symbols.next_if(|&i| func(i.token)).map(|i| i.token)
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
			Some(Stmts::If(expression, stmts)).take_if(|_| self.next_if_eq(Token::RightBrace))
		} else if self.next_if_eq(Token::Keyword(Reserved::Int))
			&& let Some(Token::Identifier(name)) =
				self.next_if(|i| matches!(i, Token::Identifier(_)))
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Decl(name))
		} else if let Some(Token::Identifier(name)) =
			self.next_if(|i| matches!(i, Token::Identifier(_)))
			&& self.next_if_eq(Token::Equal)
			&& let Some(expression) = self.expression()
			&& self.next_if_eq(Token::Semicolon)
		{
			Some(Stmts::Assignment(name, expression))
		} else {
			None
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
		match self.next_if(|i| matches!(i, Token::Identifier(_))) {
			Some(Token::Identifier(name)) => Some(DirectValue::Ident(name)),
			_ => match self.next_if(|i| matches!(i, Token::Const(_))) {
				Some(Token::Const(symbol_idx)) => {
					Some(DirectValue::Const(self.parse_const(symbol_idx)?))
				}
				_ => None,
			},
		}
	}
	fn binary_operation(&mut self) -> Option<BinaryOperation> {
		self.symbols
			.next_if(|tk| BinaryOperation::from_token(&tk.token).is_some())
			.map(|tk| BinaryOperation::from_token(&tk.token))?
	}
	/// TODO: Fix me!
	fn parse_const(&self, symbol_idx: usize) -> Option<i32> {
		let value = self
			.symbol_table
			.get_const(symbol_idx)?
			.trim_start_matches('0');
		if ('1'..='9').contains(&value.chars().nth(0)?) {
			i32::from_str_radix(value, 10).ok()
		} else {
			let radix = match value.chars().nth(0)? {
				'b' => Some(2),
				'o' => Some(8),
				'x' => Some(16),
				_ => None,
			};
			i32::from_str_radix(value, radix?).ok()
		}
	}
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Program {
	pub statement_root: Vec<Stmts>,
	pub return_value: Expression,
}

type Ident = usize;

#[derive(Clone, Debug)]
pub enum Stmts {
	If(Expression, Vec<Stmts>),
	Decl(Ident),
	Assignment(Ident, Expression),
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