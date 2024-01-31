#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	Keyword(Reserved),

	/// TODO: Change this to a string slice
	/// TODO: Add escape sequence support for String literals
	Literal(String),
	Identifier(String),
	Const(String),

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
pub enum Reserved {
	If,
	Int,
}

pub fn tokenize(input_stream: &str) -> Vec<Token> {
	let mut tokens = Vec::new();
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
				Token::Const(const_buffer)
			}
			// TODO: The numeric check here is redundant, check if compiler has optimized it
			char if is_identifier_symbol(char) && !char.is_numeric() => {
				let mut ident_buffer = char.to_string();
				while let Some(char) = stream_iter.next_if(|&i| is_identifier_symbol(i)) {
					ident_buffer.push(char);
				}
				keywords(&ident_buffer).unwrap_or(Token::Identifier(ident_buffer))
			}
			'\"' => {
				let mut literal_buffer = String::new();
				while let Some(char) = stream_iter.next_if(|&i| i != '\"') {
					literal_buffer.push(char);
				}
				stream_iter.next();
				Token::Literal(literal_buffer)
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
		tokens.push(matched_token);
	}
	tokens
}

/// TODO: should use a hash map here
fn keywords(id: &str) -> Option<Token> {
	match id {
		"if" => Some(Token::Keyword(Reserved::If)),
		"int" => Some(Token::Keyword(Reserved::Int)),
		_ => None,
	}
}

mod test {
	#[allow(unused_imports)]
	use super::{Reserved::*, Token::*, *};
	#[test]
	fn whitespace_handling() {
		assert_eq!(Vec::<Token>::new(), tokenize(""));
		assert_eq!(
			vec![
				Keyword(Int),
				Identifier("xyz".into()),
				Equal,
				Const("1".into()),
				Comma,
				Identifier("a".into()),
				Comma,
				Identifier("b".into()),
				Comma,
				Identifier("c".into()),
				Comma,
				Identifier("d".into()),
				Semicolon,
				Identifier("ident_test1234".into()),
				Semicolon,
				Const("0b1234makethisconst".into()),
				Semicolon,
				Keyword(If),
				LeftParenthesis,
				Identifier("xyz".into()),
				RightParenthesis,
				LeftBrace,
				Identifier("xyz".into()),
				Equal,
				Const("0".into()),
				Semicolon,
				RightBrace,
			],
			tokenize(
				r"
				int xyz = 1, a,b, c , d;

				ident_test1234;
				0b1234makethisconst;

				// This should be ignored / since it's a line comment
				/*
				This is a block comment * so ignore * / this as well //
				*/
					if ( xyz){
					xyz= 0 ;
				}
			"
			)
		);
	}
	#[test]
	fn operators() {
		assert_eq!(
			vec![
				Keyword(Int),
				Identifier("x".into()),
				Equal,
				Const("6".into()),
				Semicolon,
				Identifier("x".into()),
				SlashEqual,
				Const("3".into()),
				Semicolon,
				Identifier("y".into()),
				Equal,
				Literal("some literal".into()),
				Semicolon,
				Identifier("x".into()),
				PlusPlus,
				Semicolon,
				Identifier("x".into()),
				MinusMinus,
				Semicolon,
				MinusMinus,
				Identifier("x".into()),
				Semicolon,
				PlusPlus,
				Identifier("x".into()),
				Semicolon,
			],
			tokenize(
				r#"
				int x = 6;
				x /= 3;
				y = "some literal";
				x++;
				x--;
				--x;
				++x;
				"#
			)
		);
		assert_eq!(
			vec![
				Keyword(Int),
				Identifier("x".into()),
				Equal,
				Const("6".into()),
				Semicolon,
				Identifier("x".into()),
				SlashEqual,
				Const("3".into()),
				Semicolon,
				Identifier("x".into()),
				PlusPlus,
				Semicolon,
				Identifier("x".into()),
				MinusMinus,
				Semicolon,
				MinusMinus,
				Identifier("x".into()),
				Semicolon,
				PlusPlus,
				Identifier("x".into()),
				Semicolon,
			],
			tokenize(
				r"
				int x = 6;
				x /= 3;
				x++;
				x--;
				--x;
				++x;
				"
			)
		);
		assert_eq!(
			vec![
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
			],
			tokenize(
				r"
				+ - * / % ++ -- == != > < >= <= ! && || ~ & | ^ << >> = += -= *=
				/= %= &= |= ^= <<= >>= -> ? :
				"
			)
		)
	}
	#[test]
	fn string_literals() {
		assert_eq!(vec![Literal("some".into())], tokenize(r#""some""#));
		assert_eq!(
			vec![Literal("some".into())],
			tokenize(
				r#"
			"some"
				
			"#
			)
		);
	}
}
