//! Lexical Analyzer
//!
//! Call the `lexer::tokenize` function with the input source code as `&str`

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token {
	Keyword(Reserved),

	Literal(usize),
	Identifier(usize),
	Const(usize),

	// Brackets
	LeftParenthesis,
	RightParenthesis,
	LeftBrace,
	RightBrace,
	LeftSquare,
	RightSquare,
	Semicolon,

	// Operators
	// Arithmetic
	Plus,
	Minus,
	Star,
	Slash,
	Percent,
	PlusPlus,
	MinusMinus,
	// Comparison
	EqualEqual,
	BangEqual,
	Greater,
	Less,
	GreaterEqual,
	LessEqual,
	// Logical
	Bang,
	AmpAmp,
	PipePipe,
	// Bitwise
	Tilde,
	Amp,
	Pipe,
	Caret,
	LessLess,
	GreaterGreater,
	// Assignment
	Equal,
	PlusEqual,
	MinusEqual,
	StarEqual,
	SlashEqual,
	PercentEqual,
	AmpEqual,
	PipeEqual,
	CaretEqual,
	LessLessEqual,
	GreaterGreaterEqual,
	// Other
	Arrow,
	Question,
	Colon,
	Comma,

	Eof,
}

/// Tuple struct of `Token` and the corresponding `line_number: usize`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Symbol(pub Token, pub usize);
impl Symbol {
	pub fn token(&self) -> Token {
		self.0
	}
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SymbolTable {
	pub identifier: Vec<String>,
	pub consts: Vec<String>,
	pub literal: Vec<String>,
}
impl SymbolTable {
	fn add_identifier(&mut self, identifier: String) -> usize {
		self.identifier
			.iter()
			.position(|i| **i == identifier)
			.unwrap_or_else(|| {
				self.identifier.push(identifier);
				self.identifier.len() - 1
			})
	}
	fn add_consts(&mut self, consts: String) -> usize {
		self.consts
			.iter()
			.position(|i| **i == consts)
			.unwrap_or_else(|| {
				self.consts.push(consts);
				self.consts.len() - 1
			})
	}
	fn add_literal(&mut self, literal: String) -> usize {
		self.literal
			.iter()
			.position(|i| **i == literal)
			.unwrap_or_else(|| {
				self.literal.push(literal);
				self.literal.len() - 1
			})
	}
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LexerOutput {
	pub symbol_table: SymbolTable,
	pub symbol: Vec<Symbol>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reserved {
	If,
	Int,
	Return,
	While,
	Break,
	Continue,
}

pub fn tokenize(input_stream: &str) -> LexerOutput {
	let LexerOutput {
		mut symbol_table,
		mut symbol,
	} = LexerOutput::default();
	let is_identifier_symbol = |char: char| char.is_alphanumeric() || char == '_';
	let mut stream_iter = input_stream.chars().peekable();
	let mut line_number = 1;
	while let Some(current) = stream_iter.next() {
		if current == '\n' {
			line_number += 1;
		}
		if current.is_whitespace() {
			continue;
		}
		// Handle line comments
		if current == '/' && stream_iter.peek().is_some_and(|x| *x == '/') {
			while stream_iter.next_if(|x| *x != '\n').is_some() {}
			continue;
		}
		if current == '/' && stream_iter.next_if(|x| *x == '*').is_some() {
			loop {
				if let Some('*') = stream_iter.next()
					&& Some(&'/') == stream_iter.peek()
				{
					stream_iter.next();
					break;
				}
			}
			continue;
		}
		let matched_token = match current {
			char if char.is_numeric() => {
				let mut const_buffer = char.to_string();
				while let Some(char) = stream_iter.next_if(|i| i.is_alphanumeric()) {
					const_buffer.push(char);
				}
				Token::Const(symbol_table.add_consts(const_buffer))
			}
			// TODO: The numeric check here is redundant, check if compiler has optimized it
			char if is_identifier_symbol(char) && !char.is_numeric() => {
				let mut ident_buffer = char.to_string();
				while let Some(char) = stream_iter.next_if(|&i| is_identifier_symbol(i)) {
					ident_buffer.push(char);
				}
				keywords(&ident_buffer)
					.unwrap_or_else(|| Token::Identifier(symbol_table.add_identifier(ident_buffer)))
			}
			'\"' => {
				let mut literal_buffer = String::new();
				while let Some(char) = stream_iter.next_if(|&i| i != '\"') {
					literal_buffer.push(char);
				}
				stream_iter.next();
				Token::Literal(symbol_table.add_literal(literal_buffer))
			}
			'+' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::PlusEqual
				} else if stream_iter.next_if(|&i| i == '+').is_some() {
					Token::PlusPlus
				} else {
					Token::Plus
				}
			}
			'-' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::MinusEqual
				} else if stream_iter.next_if(|&i| i == '-').is_some() {
					Token::MinusMinus
				} else if stream_iter.next_if(|&i| i == '>').is_some() {
					Token::Arrow
				} else {
					Token::Minus
				}
			}
			'=' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::EqualEqual
				} else {
					Token::Equal
				}
			}
			'!' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::BangEqual
				} else {
					Token::Bang
				}
			}
			'*' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::StarEqual
				} else {
					Token::Star
				}
			}
			'/' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::SlashEqual
				} else {
					Token::Slash
				}
			}
			'%' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::PercentEqual
				} else {
					Token::Percent
				}
			}
			'<' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::LessEqual
				} else if stream_iter.next_if(|&i| i == '<').is_some() {
					if stream_iter.next_if(|&i| i == '=').is_some() {
						Token::LessLessEqual
					} else {
						Token::LessLess
					}
				} else {
					Token::Less
				}
			}
			'>' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::GreaterEqual
				} else if stream_iter.next_if(|&i| i == '>').is_some() {
					if stream_iter.next_if(|&i| i == '=').is_some() {
						Token::GreaterGreaterEqual
					} else {
						Token::GreaterGreater
					}
				} else {
					Token::Greater
				}
			}
			'&' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::AmpEqual
				} else if stream_iter.next_if(|&i| i == '&').is_some() {
					Token::AmpAmp
				} else {
					Token::Amp
				}
			}
			'|' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::PipeEqual
				} else if stream_iter.next_if(|&i| i == '|').is_some() {
					Token::PipePipe
				} else {
					Token::Pipe
				}
			}
			'^' => {
				if stream_iter.next_if(|&i| i == '=').is_some() {
					Token::CaretEqual
				} else {
					Token::Caret
				}
			}
			'?' => Token::Question,
			':' => Token::Colon,
			'~' => Token::Tilde,
			',' => Token::Comma,
			';' => Token::Semicolon,
			'(' => Token::LeftParenthesis,
			')' => Token::RightParenthesis,
			'{' => Token::LeftBrace,
			'}' => Token::RightBrace,
			'[' => Token::LeftSquare,
			']' => Token::RightSquare,
			x => panic!("{x} at line#{line_number}"),
		};
		symbol.push(Symbol(matched_token, line_number));
	}
	symbol.push(Symbol(Token::Eof, line_number));
	LexerOutput {
		symbol_table,
		symbol,
	}
}

fn keywords(id: &str) -> Option<Token> {
	match id {
		"if" => Some(Token::Keyword(Reserved::If)),
		"int" => Some(Token::Keyword(Reserved::Int)),
		"return" => Some(Token::Keyword(Reserved::Return)),
		"while" => Some(Token::Keyword(Reserved::While)),
		"break" => Some(Token::Keyword(Reserved::Break)),
		"continue" => Some(Token::Keyword(Reserved::Continue)),
		_ => None,
	}
}

mod test {
	#[allow(unused_imports)]
	use super::*;
	#[test]
	fn comments() {
		assert_eq!(
			LexerOutput {
				symbol: vec![Symbol(Token::Eof, 1)],
				..Default::default()
			},
			tokenize("")
		);
		assert_eq!(
			LexerOutput {
				symbol: vec![Symbol(Token::Eof, 1)],
				..Default::default()
			},
			tokenize("//")
		);
		assert_eq!(
			LexerOutput {
				symbol: vec![Symbol(Token::Eof, 3)],
				..Default::default()
			},
			tokenize(
				r"
				/*
					Some text
				*/
				"
			)
		);
	}
	#[test]
	fn program() {
		use Reserved::*;
		use Token::*;
		assert_eq!(
			LexerOutput {
				symbol_table: SymbolTable {
					identifier: vec!["first".into(), "second".into(), "i".into(), "temp".into()],
					consts: vec!["1".into(), "0".into(), "10".into()],
					..Default::default()
				},
				symbol: vec![
					Symbol(Keyword(Int), 3),
					Symbol(Identifier(0), 3),
					Symbol(Semicolon, 3),
					Symbol(Keyword(Int), 4),
					Symbol(Identifier(1), 4),
					Symbol(Semicolon, 4),
					Symbol(Keyword(Int), 5),
					Symbol(Identifier(2), 5),
					Symbol(Semicolon, 5),
					Symbol(Identifier(2), 6),
					Symbol(Equal, 6),
					Symbol(Const(0), 6),
					Symbol(Semicolon, 6),
					Symbol(Identifier(0), 7),
					Symbol(Equal, 7),
					Symbol(Const(1), 7),
					Symbol(Semicolon, 7),
					Symbol(Identifier(1), 8),
					Symbol(Equal, 8),
					Symbol(Const(0), 8),
					Symbol(Semicolon, 8),
					Symbol(Keyword(While), 9),
					Symbol(LeftParenthesis, 9),
					Symbol(Identifier(2), 9),
					Symbol(Less, 9),
					Symbol(Const(2), 9),
					Symbol(RightParenthesis, 9),
					Symbol(LeftBrace, 9),
					Symbol(Keyword(Int), 10),
					Symbol(Identifier(3), 10),
					Symbol(Semicolon, 10),
					Symbol(Identifier(3), 11),
					Symbol(Equal, 11),
					Symbol(Identifier(1), 11),
					Symbol(Semicolon, 11),
					Symbol(Identifier(1), 12),
					Symbol(Equal, 12),
					Symbol(Identifier(0), 12),
					Symbol(Plus, 12),
					Symbol(Identifier(3), 12),
					Symbol(Semicolon, 12),
					Symbol(Identifier(0), 13),
					Symbol(Equal, 13),
					Symbol(Identifier(3), 13),
					Symbol(Semicolon, 13),
					Symbol(Identifier(2), 14),
					Symbol(Equal, 14),
					Symbol(Identifier(2), 14),
					Symbol(Plus, 14),
					Symbol(Const(0), 14),
					Symbol(Semicolon, 14),
					Symbol(RightBrace, 15),
					Symbol(Keyword(Return), 16),
					Symbol(Identifier(1), 16),
					Symbol(Semicolon, 16),
					Symbol(Eof, 17)
				]
			},
			tokenize(
				r"
// Compute the 10th fibonnaci number
int first;
int second;
int i;
i = 1;
first = 0;
second = 1;
while (i < 10) {
	int temp;
	temp = second;
	second = first + temp;
	first = temp;
	i = i + 1;
}
return second;
",
			)
		);
	}
}
