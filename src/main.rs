#![feature(let_chains)]

mod lexer;
use lexer::tokenize;

const TEST_PROGRAM: &'static str = r"
int xyz = 1;
if (xyz){
	xyz=0;
}
";

fn main() {
	println!("{TEST_PROGRAM}");
	println!("{:?}", tokenize(&TEST_PROGRAM))
}
