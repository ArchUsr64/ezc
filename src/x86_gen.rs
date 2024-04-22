//! x86 backend
use std::collections::HashMap;
use std::fmt::Write;

use crate::{
	parser::{self, BinaryOperation},
	tac_gen::{self, Function, Ident, Operand, RValue},
};

const PRELUDE: &str = r".intel_mnemonic
.intel_syntax
.text
";

/// stack offset to access arguments, comprises of 8 bytes for the return address
/// of the caller and 8 bytes for caller's `rbp`
const ARGUMENTS_STACK_OFFSET: usize = 16;

pub fn x86_gen(
	tac_instruction: Vec<tac_gen::Function>,
	ident_table: parser::IdentNameTable,
) -> String {
	let mut res = PRELUDE.to_string();

	res += tac_instruction
		.iter()
		.fold(String::new(), |mut out, Function { id, .. }| {
			let _ = write!(
				out,
				"\n.global {func_name}\n.type {func_name}, @function",
				func_name = ident_table.0[*id]
			);
			out
		})
		.as_str();

	for tac_gen::Function {
		id: func_id,
		instructions,
	} in tac_instruction.iter()
	{
		let func_name = ident_table.0[*func_id].as_str();
		res += format!(
			r"
{func_name}:
F{func_id}:
	push %rbp
	mov %rbp, %rsp
"
		)
		.as_str();
		let mut if_count = 0;
		let mut goto_count = 0;
		// Stores the list of instructions
		let mut if_jumps = Vec::new();
		let mut goto_jumps = Vec::new();
		let mut allocator = StackAllocator::default();
		use tac_gen::Instruction;
		for (i, instruction) in instructions.iter().enumerate() {
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
		let mut asm_instructions: Vec<Vec<String>> = instructions
			.iter()
			.enumerate()
			.map(|(i, tac)| {
				let mut asm = Vec::new();
				if log::log_enabled!(log::Level::Debug) {
					asm.push(format!("\n# {i}: {tac:?}"));
				}
				asm.append(&mut match tac {
					Instruction::ArrayWrite(name, index, r_val) => vec![
						format!("mov %edi, {}", allocator.parse_operand(*index)),
						format!("shl %rdi"),
						format!("shl %rdi"),
						format!("mov %rsi, %rbp"),
						format!("sub %rsi, {}", allocator.ident_table.get(&name).unwrap()),
						format!("add %rsi, %rdi"),
						format!("mov %eax, {}", allocator.parse_operand(*r_val)),
						format!("mov DWORD PTR [%rsi], %eax"),
					],
					Instruction::ArrayAlloc(name, size) => {
						allocator.array_alloc(*dbg!(name), *size);
						Vec::new()
					}
					Instruction::Return(op) => vec![
						format!("mov %eax, {}", allocator.parse_operand(*op)),
						format!("jmp END_{func_id}"),
					],
					Instruction::Push(op) => vec![
						format!("mov %eax, {}", allocator.parse_operand(*op)),
						format!("sub %rsp, {}", INTEGER_SIZE),
						format!("mov DWORD PTR [%rsp], %eax"),
					],
					Instruction::Expression(op, r_value) => allocator.expression_gen(*op, *r_value),
					Instruction::Ifz(op, _) => {
						if_count += 1;
						vec![
							format!("cmp {}, 0", allocator.parse_operand(*op)),
							format!("je L{}_{func_id}", if_count - 1),
						]
					}
					Instruction::Goto(_) => {
						goto_count += 1;
						vec![format!("jmp G{}_{func_id}", goto_count - 1)]
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
					asm.insert(0, format!("L{label_id}_{func_id}:"));
				} else if let Some(last) = asm_instructions.last_mut() {
					last.push(format!("L{label_id}_{func_id}:"));
				}
			});
		goto_jumps
			.iter()
			.enumerate()
			.for_each(|(label_id, &tac_index)| {
				let tac_index = tac_index as usize;
				if let Some(asm) = asm_instructions.get_mut(tac_index) {
					asm.insert(0, format!("G{label_id}_{func_id}:"));
				} else if let Some(last) = asm_instructions.last_mut() {
					last.push(format!("G{label_id}_{func_id}:"));
				};
			});
		res += format!("	sub %rsp, {}\n", allocator.stack_usage).as_str();
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
		res += format!(
			r"END_{func_id}:
	add %rsp, {}
	pop %rbp
	ret
",
			allocator.stack_usage
		)
		.as_str();
	}
	res
}

const INTEGER_SIZE: usize = 4;

#[derive(Debug, Default)]
struct StackAllocator {
	stack_usage: usize,
	ident_table: HashMap<Ident, usize>,
	arguments_size: usize,
	temporary_offset: HashMap<usize, usize>,
}
impl StackAllocator {
	fn parse_operand(&mut self, operand: Operand) -> String {
		match operand {
			Operand::Ident(Ident::Parameter(offset)) => {
				format!(
					"DWORD PTR [%rbp + {}]",
					ARGUMENTS_STACK_OFFSET + offset * INTEGER_SIZE
				)
			}
			Operand::Ident(ident) => {
				let offset = *self.ident_table.get(&ident).unwrap_or_else(|| {
					self.stack_usage += INTEGER_SIZE;
					&self.stack_usage
				});
				self.ident_table.insert(ident, offset);
				format!("DWORD PTR [%rbp - {offset}]")
			}
			Operand::Temporary(id) => {
				let offset = *self.temporary_offset.get(&id).unwrap_or_else(|| {
					self.stack_usage += INTEGER_SIZE;
					&self.stack_usage
				});
				self.temporary_offset.insert(id, offset);
				format!("DWORD PTR [%rbp - {offset}]")
			}
			Operand::Immediate(val) => val.to_string(),
		}
	}
	fn array_alloc(&mut self, name: Ident, size: u32) {
		self.stack_usage += INTEGER_SIZE * size as usize;
		self.ident_table.insert(name, self.stack_usage);
	}
	fn expression_gen(&mut self, l_value: Operand, r_value: RValue) -> Vec<String> {
		match r_value {
			RValue::ArrayAccess(ident, index) => {
				vec![
					format!("mov %edi, {}", self.parse_operand(index)),
					format!("shl %rdi"),
					format!("shl %rdi"),
					format!("mov %rsi, %rbp"),
					format!(
						"sub %rsi, {}",
						dbg!(&self.ident_table).get(&dbg!(ident)).unwrap()
					),
					format!("add %rsi, %rdi"),
					format!("mov %eax, DWORD PTR [%rsi]"),
					format!("mov {}, %eax", self.parse_operand(l_value)),
				]
			}
			RValue::Assignment(Operand::Immediate(val)) => {
				vec![format!("mov {}, {}", self.parse_operand(l_value), val)]
			}
			RValue::Assignment(r_value) => vec![
				format!("mov %eax, {}", self.parse_operand(r_value)),
				format!("mov {}, %eax", self.parse_operand(l_value)),
			],
			RValue::FuncCall(func_id, arg_count) => {
				self.arguments_size = 0;
				vec![
					format!("call F{func_id}"),
					format!("mov {}, %eax", self.parse_operand(l_value)),
					format!("add %rsp, {}", arg_count * INTEGER_SIZE),
				]
			}
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
