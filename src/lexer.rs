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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
	pub token: Token,
	pub line_number: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolTable {
	identifier: Vec<String>,
	consts: Vec<String>,
	literal: Vec<String>,
}
impl SymbolTable {
	pub fn get_const(&self, index: usize) -> Option<&String> {
		self.consts.get(index)
	}
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

#[derive(Clone, Debug, PartialEq)]
pub struct LexerOutput {
	pub symbol_table: SymbolTable,
	pub symbol: Vec<Symbol>,
}
impl LexerOutput {
	fn new() -> Self {
		Self {
			symbol_table: SymbolTable {
				identifier: Vec::new(),
				consts: Vec::new(),
				literal: Vec::new(),
			},
			symbol: Vec::new(),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reserved {
	If,
	Int,
	Return,
}

pub fn tokenize(input_stream: &str) -> LexerOutput {
	let LexerOutput {
		mut symbol_table,
		mut symbol,
	} = LexerOutput::new();
	let is_identifier_symbol = |char: char| char.is_alphanumeric() || char == '_';
	let mut stream_iter = input_stream.chars().peekable();
	let mut line_number = 0;
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
			line_number += 1;
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
			x => todo!("{x} at line#{line_number}"),
		};
		symbol.push(Symbol {
			token: matched_token,
			line_number,
		});
	}
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
		_ => None,
	}
}
