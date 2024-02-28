use std::collections::HashMap;

use crate::{
	parser::BinaryOperation,
	tac_gen::{self, BindedIdent, Operand, RValue},
};

const PRELUDE: &'static str = r"
.intel_mnemonic
.intel_syntax
.text

.globl test_func
.type test_func, @function

test_func:
";

const PROLOGUE: &'static str = r"
	push %rbp
	mov %rbp, %rsp
";

const EPILOGUE: &'static str = r"
	pop %rbp
	ret
";

pub fn x86_gen(tac_instruction: Vec<tac_gen::Instruction>) -> String {
	let mut res = PRELUDE.to_string();
	let mut label_count = 0;
	// Map from the instruction index to the label count
	let mut label_map: HashMap<usize, usize> = HashMap::new();
	res.push_str(PROLOGUE);
	let mut allocator = StackAllocator::new();
	for (i, instruction) in tac_instruction.iter().enumerate() {
		use tac_gen::Instruction;
		let mut instructions = match instruction {
			Instruction::Return(op) => {
				vec![format!("mov %eax, {}", allocator.parse_operand(*op))]
			}
			Instruction::Expression(op, r_value) => allocator.expression_gen(*op, *r_value),
			Instruction::Ifz(op, offset) => {
				let label_id = if let Some(label_index) = label_map.get(&(i + *offset)) {
					*label_index
				} else {
					label_count += 1;
					label_map.insert(i + *offset, label_count);
					label_count
				};
				vec![
					format!("cmp {}, 0", allocator.parse_operand(*op)),
					format!("je L{label_id}"),
				]
			}
		};
		if let Some(label_index) = label_map.get(&i) {
			instructions.push(format!("L{label_index}:"));
		}
		instructions.iter_mut().for_each(|i| {
			res += format!("	{i}\n").as_str();
			i.insert_str(0, "	");
			i.push('\n')
		});
	}
	res.push_str(EPILOGUE);
	res
}

#[derive(Debug)]
struct StackAllocator {
	stack_usage: usize,
	ident_table: HashMap<BindedIdent, usize>,
	temporary_var_table: HashMap<usize, usize>,
}

impl StackAllocator {
	fn new() -> Self {
		Self {
			stack_usage: 0,
			ident_table: HashMap::new(),
			temporary_var_table: HashMap::new(),
		}
	}
	fn parse_operand(&mut self, operand: Operand) -> String {
		match operand {
			Operand::Ident(ident) => {
				let offset = *self.ident_table.get(&ident).unwrap_or_else(|| {
					self.stack_usage += 4;
					&self.stack_usage
				});
				self.ident_table.insert(ident, offset);
				format!("DWORD PTR [%rbp - {offset}]")
			}
			Operand::Temporary(index) => {
				let offset = *self.temporary_var_table.get(&index).unwrap_or_else(|| {
					self.stack_usage += 4;
					&self.stack_usage
				});
				self.temporary_var_table.insert(index, offset);
				format!("DWORD PTR [%rbp - {offset}]")
			}
			Operand::Immediate(val) => val.to_string(),
		}
	}
	fn expression_gen(&mut self, op: Operand, r_value: RValue) -> Vec<String> {
		match r_value {
			RValue::Assignment(Operand::Immediate(val)) => {
				vec![format!("mov {}, {}", self.parse_operand(op), val)]
			}
			RValue::Assignment(r_value) => {
				vec![
					format!("mov %eax, {}", self.parse_operand(r_value)),
					format!("mov {}, %eax", self.parse_operand(op)),
				]
			}
			RValue::Operation(lhs, operation, rhs) => {
				enum Operation {
					Arithmetic,
					Conditional,
				}
				let (operation, op_code) = match operation {
					BinaryOperation::Add => (Operation::Arithmetic, "add"),
					BinaryOperation::Sub => (Operation::Arithmetic, "sub"),
					BinaryOperation::Mul => (Operation::Arithmetic, "mul"),
					BinaryOperation::Div => (Operation::Arithmetic, "div"),
					BinaryOperation::And => (Operation::Arithmetic, "and"),
					BinaryOperation::Or => (Operation::Arithmetic, "or"),
					BinaryOperation::Xor => (Operation::Arithmetic, "xor"),
					BinaryOperation::Less => (Operation::Conditional, "setl"),
					BinaryOperation::LessEqual => (Operation::Conditional, "setle"),
					BinaryOperation::Greater => (Operation::Conditional, "setg"),
					BinaryOperation::GreaterEqual => (Operation::Conditional, "setge"),
					BinaryOperation::Equal => (Operation::Conditional, "sete"),
					BinaryOperation::NotEqual => (Operation::Conditional, "setne"),
				};
				match operation {
					Operation::Arithmetic => vec![
						format!("mov %eax, {}", self.parse_operand(lhs)),
						format!("{} %eax, {}", op_code, self.parse_operand(rhs)),
						format!("mov {}, %eax", self.parse_operand(op)),
					],
					Operation::Conditional => {
						vec![
							format!("mov %eax, {}", self.parse_operand(lhs)),
							format!("cmp %eax, {}", self.parse_operand(rhs)),
							format!("{op_code} %al"),
							format!("and %al, 1"),
							format!("movzx %eax, %al"),
							format!("mov {}, %eax", self.parse_operand(op)),
						]
					}
				}
			}
		}
	}
}
