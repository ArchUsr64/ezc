#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	/// TODO: Change this to a string slice
	Identifier(String),
	Const(String),
	If,
	LeftParenthesis,
	RightParenthesis,
	LeftBrace,
	RightBrace,
	Semicolon,
	Equal,
	Int,
}

pub fn tokenize(input_stream: &str) -> Vec<Token> {
	let mut tokens = Vec::new();
	let is_identifier_symbol = |char: char| char.is_alphanumeric() || char == '_';
	let mut stream_iter = input_stream.chars().peekable();
	while let Some(current) = stream_iter.next() {
		if current.is_whitespace() {
			continue;
		}
		let matched_token = match current {
			char if char.is_numeric() => {
				let mut const_buffer = char.to_string();
				while let Some(char) = stream_iter.next_if(|i| i.is_numeric()) {
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
			'=' => Token::Equal,
			';' => Token::Semicolon,
			'(' => Token::LeftParenthesis,
			')' => Token::RightParenthesis,
			'{' => Token::LeftBrace,
			'}' => Token::RightBrace,
			x => todo!("{}", x),
		};
		tokens.push(matched_token);
	}
	tokens
}

/// TODO: should use a hash map here
fn keywords(id: &str) -> Option<Token> {
	match id {
		"if" => Some(Token::If),
		"int" => Some(Token::Int),
		_ => None,
	}
}

#[test]
fn lexer() {
	use super::*;
	use Token::*;
	assert_eq!(Vec::<Token>::new(), tokenize(""));
	assert_eq!(
		vec![
			Int,
			Identifier("xyz".to_string()),
			Equal,
			Const("1".to_string()),
			Semicolon,
			If,
			LeftParenthesis,
			Identifier("xyz".to_string()),
			RightParenthesis,
			LeftBrace,
			Identifier("xyz".to_string()),
			Equal,
			Const("0".to_string()),
			Semicolon,
			RightBrace,
		],
		tokenize(
			r"
int xyz = 1;
if (xyz){
	xyz=0;
}
			"
		)
	);
}
