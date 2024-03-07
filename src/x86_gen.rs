//! x86 backend
use std::collections::HashMap;

use crate::{
	parser::BinaryOperation,
	tac_gen::{self, BindedIdent, Operand, RValue},
};

const PRELUDE: &str = r"
.intel_mnemonic
.intel_syntax
.text
.global test_func
.type test_func, @function

test_func:
";

const PROLOGUE: &str = r"
	push %rbp
	mov %rbp, %rsp
";

const EPILOGUE: &str = r"
	END:
	pop %rbp
	ret
";

pub fn x86_gen(tac_instruction: Vec<tac_gen::Instruction>) -> String {
	let mut res = PRELUDE.to_string();
	let mut if_count = 0;
	let mut goto_count = 0;
	// Stores the list of instructions
	let mut if_jumps = Vec::new();
	let mut goto_jumps = Vec::new();
	res.push_str(PROLOGUE);
	let mut allocator = StackAllocator::new();
	use tac_gen::Instruction;
	for (i, instruction) in tac_instruction.iter().enumerate() {
		match instruction {
			Instruction::Goto(offset) => {
				goto_jumps.push(i as isize + *offset);
			}
			Instruction::Ifz(_, offset) => {
				if_jumps.push(i + *offset);
			}
			_ => continue,
		}
	}
	let mut asm_instructions: Vec<Vec<String>> = tac_instruction
		.iter()
		.enumerate()
		.map(|(i, tac)| {
			let mut asm = Vec::new();
			if log::log_enabled!(log::Level::Debug) {
				asm.push(format!("\n# {i}: {tac:?}"));
			}
			asm.append(&mut match tac {
				Instruction::Return(op) => vec![
					format!("mov %eax, {}", allocator.parse_operand(*op)),
					format!("jmp END"),
				],
				Instruction::Expression(op, r_value) => allocator.expression_gen(*op, *r_value),
				Instruction::Ifz(op, _) => {
					if_count += 1;
					vec![
						format!("cmp {}, 0", allocator.parse_operand(*op)),
						format!("je L{}", if_count - 1),
					]
				}
				Instruction::Goto(_) => {
					goto_count += 1;
					vec![format!("jmp G{}", goto_count - 1)]
				}
			});
			asm
		})
		.collect();
	if_jumps
		.iter()
		.enumerate()
		.for_each(|(label_id, &tac_index)| {
			if let Some(asm) = asm_instructions.get_mut(tac_index) {
				asm.insert(0, format!("L{label_id}:"));
			} else if let Some(last) = asm_instructions.last_mut() {
				last.push(format!("L{label_id}:"));
			}
		});
	goto_jumps
		.iter()
		.enumerate()
		.for_each(|(label_id, &tac_index)| {
			let tac_index = tac_index as usize;
			if let Some(asm) = asm_instructions.get_mut(tac_index) {
				asm.insert(0, format!("G{label_id}:"));
			} else if let Some(last) = asm_instructions.last_mut() {
				last.push(format!("G{label_id}:"));
			};
		});
	res.push_str(
		asm_instructions
			.iter()
			.flat_map(|asm_set| {
				asm_set
					.iter()
					.map(|instruction| format!("\t{instruction}\n"))
			})
			.collect::<String>()
			.as_str(),
	);
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
	fn expression_gen(&mut self, l_value: Operand, r_value: RValue) -> Vec<String> {
		match r_value {
			RValue::Assignment(Operand::Immediate(val)) => {
				vec![format!("mov {}, {}", self.parse_operand(l_value), val)]
			}
			RValue::Assignment(r_value) => vec![
				format!("mov %eax, {}", self.parse_operand(r_value)),
				format!("mov {}, %eax", self.parse_operand(l_value)),
			],
			RValue::Operation(lhs, operation, rhs) => {
				enum Operation {
					Arithmetic(&'static str),
					Conditional(&'static str),
					// These require special code gen
					Mul,
					Div,
					Mod,
				}
				let operation = match operation {
					BinaryOperation::Add => Operation::Arithmetic("add"),
					BinaryOperation::Sub => Operation::Arithmetic("sub"),
					BinaryOperation::And => Operation::Arithmetic("and"),
					BinaryOperation::Or => Operation::Arithmetic("or"),
					BinaryOperation::Xor => Operation::Arithmetic("xor"),
					BinaryOperation::Less => Operation::Conditional("setl"),
					BinaryOperation::LessEqual => Operation::Conditional("setle"),
					BinaryOperation::Greater => Operation::Conditional("setg"),
					BinaryOperation::GreaterEqual => Operation::Conditional("setge"),
					BinaryOperation::Equal => Operation::Conditional("sete"),
					BinaryOperation::NotEqual => Operation::Conditional("setne"),
					BinaryOperation::Mul => Operation::Mul,
					BinaryOperation::Div => Operation::Div,
					BinaryOperation::Mod => Operation::Mod,
				};
				match operation {
					Operation::Arithmetic(op_code) => vec![
						format!("mov %eax, {}", self.parse_operand(lhs)),
						format!("{} %eax, {}", op_code, self.parse_operand(rhs)),
						format!("mov {}, %eax", self.parse_operand(l_value)),
					],
					Operation::Conditional(op_code) => vec![
						format!("mov %eax, {}", self.parse_operand(lhs)),
						format!("cmp %eax, {}", self.parse_operand(rhs)),
						format!("{op_code} %al"),
						format!("and %al, 1"),
						format!("movzx %eax, %al"),
						format!("mov {}, %eax", self.parse_operand(l_value)),
					],
					Operation::Mul => vec![
						format!("mov %eax, {}", self.parse_operand(lhs),),
						format!("mov %ecx, {}", self.parse_operand(rhs),),
						format!("imul %eax, %ecx"),
						format!("mov {}, %eax", self.parse_operand(l_value)),
					],
					Operation::Div => vec![
						format!("mov %eax, {}", self.parse_operand(lhs),),
						format!("mov %ecx, {}", self.parse_operand(rhs),),
						format!("cdq"),
						format!("idiv %ecx"),
						format!("mov {}, %eax", self.parse_operand(l_value)),
					],
					Operation::Mod => vec![
						format!("mov %eax, {}", self.parse_operand(lhs),),
						format!("mov %ecx, {}", self.parse_operand(rhs),),
						format!("cdq"),
						format!("idiv %ecx"),
						format!("mov {}, %edx", self.parse_operand(l_value)),
					],
				}
			}
		}
	}
}
