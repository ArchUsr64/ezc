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

// TODO: Error `Symbol` instead for better debuggability
pub fn parse(lexer_output: LexerOutput) -> Result<Program, Option<Token>> {
	let LexerOutput {
		symbol_table,
		symbol,
	} = lexer_output;
	let mut parser = Parser {
		tokens: symbol.iter().map(|Symbol { token, .. }| *token).peekable(),
		symbol_table,
	};
	let mut statement_root = Vec::new();
	while let Some(stmt) = parser.stmts() {
		statement_root.push(stmt);
	}
	Ok(Program {
		statement_root,
		return_value: parser
			.return_value()
			.ok_or_else(|| parser.tokens.peek().map(|i| *i))?,
	})
}

struct Parser<I: Iterator<Item = Token>> {
	tokens: Peekable<I>,
	symbol_table: SymbolTable,
}
impl<I: Iterator<Item = Token>> Parser<I> {
	#[inline]
	fn next_eq(&mut self, needle: Token) -> bool {
		self.tokens.next_if_eq(&needle).is_some()
	}
	fn stmts(&mut self) -> Option<Stmts> {
		if self.next_eq(Token::Keyword(Reserved::If)) && self.next_eq(Token::LeftParenthesis) {
			let expression = self.expression()?;
			if !(self.next_eq(Token::RightParenthesis) && self.next_eq(Token::LeftBrace)) {
				return None;
			};
			let mut stmts = Vec::new();
			while let Some(stmt) = self.stmts() {
				stmts.push(stmt);
			}
			Some(Stmts::If(expression, stmts)).take_if(|_| self.next_eq(Token::RightBrace))
		} else if self.next_eq(Token::Keyword(Reserved::Int))
			&& let Some(Token::Identifier(name)) =
				self.tokens.next_if(|i| matches!(i, Token::Identifier(_)))
			&& self.next_eq(Token::Semicolon)
		{
			Some(Stmts::Decl(name))
		} else if let Some(Token::Identifier(name)) =
			self.tokens.next_if(|i| matches!(i, Token::Identifier(_)))
			&& self.next_eq(Token::Equal)
			&& let Some(expression) = self.expression()
			&& self.next_eq(Token::Semicolon)
		{
			Some(Stmts::Assignment(name, expression))
		} else {
			None
		}
	}
	fn return_value(&mut self) -> Option<Expression> {
		self.next_eq(Token::Keyword(Reserved::Return))
			.then_some(())
			.and_then(|_| self.expression())
			.take_if(|_| self.tokens.next_if_eq(&Token::Semicolon).is_some())
	}
	fn expression(&mut self) -> Option<Expression> {
		if let Some(l_value) = self.direct_value() {
			if let Some(binary_operation) = self.binary_operation()
				&& let Some(r_value) = self.direct_value()
			{
				return Some(Expression::BinaryExpression(
					l_value,
					binary_operation,
					r_value,
				));
			}
			return Some(Expression::DirectValue(l_value));
		}
		None
	}
	fn direct_value(&mut self) -> Option<DirectValue> {
		match self.tokens.next_if(|i| matches!(i, Token::Identifier(_))) {
			Some(Token::Identifier(name)) => Some(DirectValue::Ident(name)),
			_ => match self.tokens.next_if(|i| matches!(i, Token::Const(_))) {
				Some(Token::Const(symbol_idx)) => {
					Some(DirectValue::Const(self.parse_const(symbol_idx)?))
				}
				_ => None,
			},
		}
	}
	fn binary_operation(&mut self) -> Option<BinaryOperation> {
		self.tokens
			.next_if(|tk| BinaryOperation::from_token(tk).is_some())
			.map(|tk| BinaryOperation::from_token(&tk))?
	}
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
	statement_root: Vec<Stmts>,
	return_value: Expression,
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
