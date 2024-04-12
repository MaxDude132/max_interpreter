use core::panic;

use crate::value::{print_value, Value};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    OpConstant,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpNot,
    OpTrue,
    OpFalse,
    OpNone,  // TODO: Remove eventually
    OpPrint, // TODO: Remove eventually
    OpEqual,
    OpNotEqual,
    OpGreater,
    OpGreaterEqual,
    OpLess,
    OpLessEqual,
    OpReturn,
    OpSet,
    OpGet,
    OpEol,
    OpEof,
    OpPop,
    OpJumpIfTrue,
    OpJumpIfFalse,
    OpJump,
    OpLoop,
    Number(usize),
}

impl OpCode {
    pub fn as_number(&self) -> usize {
        match self {
            OpCode::Number(n) => *n,
            _ => panic!("Expected OpCode to be a number"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    lines: Vec<usize>,
    pub constants: Vec<Value>,
    pub had_error: bool,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
            had_error: false,
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);

        let lines_len = self.lines.len();
        if lines_len > 1 && self.lines[lines_len - 2] == line {
            self.lines[lines_len - 1] += 1;
        } else {
            self.lines.push(line);
            self.lines.push(1);
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_line(&self, index: usize) -> usize {
        let mut line = 0;

        for i in (0..self.lines.len()).step_by(2) {
            line += self.lines[i + 1];
            if line - 1 >= index {
                return self.lines[i];
            }
        }
        panic!("Index out of bounds")
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut skip_next: usize = 0;
        for (index, byte) in self.code.iter().enumerate() {
            if skip_next > 0 {
                skip_next -= 1;
                continue;
            }
            skip_next = self.disassemble_instruction(byte, index);
        }
    }

    pub fn disassemble_instruction(&self, byte: &OpCode, index: usize) -> usize {
        print!("{:04} ", index);
        let line = self.get_line(index);
        if index > 0 && line == self.get_line(index - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", line);
        }

        match byte {
            OpCode::OpConstant => {
                self.constant_instruction("OP_CONSTANT", index);
                1
            }
            OpCode::OpAdd => {
                println!("OP_ADD");
                0
            }
            OpCode::OpSubtract => {
                println!("OP_SUBTRACT");
                0
            }
            OpCode::OpMultiply => {
                println!("OP_MULTIPLY");
                0
            }
            OpCode::OpDivide => {
                println!("OP_DIVIDE");
                0
            }
            OpCode::OpTrue => {
                println!("OP_TRUE");
                0
            }
            OpCode::OpFalse => {
                println!("OP_FALSE");
                0
            }
            OpCode::OpNone => {
                println!("OP_NONE");
                0
            }
            OpCode::OpPrint => {
                println!("OP_PRINT");
                0
            }
            OpCode::OpNot => {
                println!("OP_NOT");
                0
            }
            OpCode::OpNegate => {
                println!("OP_NEGATE");
                0
            }
            OpCode::OpEqual => {
                println!("OP_EQUAL");
                0
            }
            OpCode::OpNotEqual => {
                println!("OP_NOT_EQUAL");
                0
            }
            OpCode::OpGreater => {
                println!("OP_GREATER");
                0
            }
            OpCode::OpGreaterEqual => {
                println!("OP_GREATER_EQUAL");
                0
            }
            OpCode::OpLess => {
                println!("OP_LESS");
                0
            }
            OpCode::OpLessEqual => {
                println!("OP_LESS_EQUAL");
                0
            }
            OpCode::OpReturn => {
                println!("OP_RETURN");
                0
            }
            OpCode::OpSet => {
                self.byte_instruction("OP_SET", index);
                1
            }
            OpCode::OpGet => {
                self.byte_instruction("OP_GET", index);
                1
            }
            OpCode::OpEol => {
                println!("OP_EOL");
                0
            }
            OpCode::OpEof => {
                println!("OP_EOF");
                0
            }
            OpCode::OpPop => {
                println!("OP_POP");
                0
            }
            OpCode::OpJumpIfTrue => {
                self.byte_instruction("OP_JUMP_IF_TRUE", index);
                1
            }
            OpCode::OpJumpIfFalse => {
                self.byte_instruction("OP_JUMP_IF_FALSE", index);
                1
            }
            OpCode::OpJump => {
                self.byte_instruction("OP_JUMP", index);
                1
            }
            OpCode::OpLoop => {
                self.byte_instruction("OP_LOOP", index);
                1
            }
            _ => panic!(
                "Unhandled value in chunk: {:?}. Here's the whole sequence: {:?}",
                byte, self.code
            ),
        }
    }

    fn byte_instruction(&self, op_code: &str, index: usize) {
        print!("{:30}", op_code);
        let slot = self.code[index + 1];
        println!("{:?}", slot);
    }

    fn constant_instruction(&self, op_code: &str, index: usize) {
        let constant = self.code[index + 1];
        let value = match constant {
            OpCode::Number(index) => self.constants[index].clone(),
            _ => panic!("Expected constant to be a number"),
        };
        print!("{:30}", op_code);
        print_value(value);
        println!();
    }
}
