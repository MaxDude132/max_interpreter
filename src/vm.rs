use crate::common::DEBUG_TRACE_EXECUTION;
use crate::compiler::{Compiler, FunctionType};
use crate::object::ObjFunction;
use crate::{
    chunk::OpCode,
    value::{print_value, Value},
};

macro_rules! binary_op {
    ($frame:expr, $vm:expr, $operator:tt) => {
        {
            let b = $frame.slots.pop().unwrap();
            let a = $frame.slots.pop().unwrap();
            let val = a $operator b;
            match val {
                Ok(val) => $frame.slots.push(val),
                Err(message) => {
                    $vm.runtime_error($frame, &message);
                    return InterpretResult::RuntimeError;
                }
            }
        }
    };
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

#[derive(Clone)]
struct CallFrame {
    ip: usize,
    function: ObjFunction,
    slots: Vec<Value>,
}

pub struct VM {
    frames: Vec<CallFrame>,
}

impl VM {
    pub fn new() -> VM {
        VM { frames: Vec::new() }
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new();
        let function = compiler.compile(source);
        if function.had_error() {
            eprintln!("Errors were found at compile time.");
            return InterpretResult::CompileError;
        }

        let frame = {
            CallFrame {
                ip: 0,
                function,
                slots: Vec::new(),
            }
        };

        self.frames.push(frame);

        let result = self.run();
        return result;
    }

    fn run(&mut self) -> InterpretResult {
        let mut frame = self.frames.last_mut().unwrap().clone();
        loop {
            let instruction = self.read_byte(&mut frame);
            if DEBUG_TRACE_EXECUTION {
                frame
                    .function
                    .chunk
                    .disassemble_instruction(&instruction, frame.ip - 1);
            }

            match instruction {
                OpCode::OpConstant => {
                    let constant = self.read_constant(&mut frame);
                    frame.slots.push(constant);
                }
                OpCode::OpAdd => binary_op!(&mut frame, self, +),
                OpCode::OpSubtract => binary_op!(&mut frame, self, -),
                OpCode::OpMultiply => binary_op!(&mut frame, self, *),
                OpCode::OpDivide => binary_op!(&mut frame, self, /),
                OpCode::OpEqual => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a == b { Value::True } else { Value::False });
                }
                OpCode::OpNotEqual => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a != b { Value::True } else { Value::False });
                }
                OpCode::OpGreater => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a > b { Value::True } else { Value::False });
                }
                OpCode::OpGreaterEqual => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a >= b { Value::True } else { Value::False });
                }
                OpCode::OpLess => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a < b { Value::True } else { Value::False });
                }
                OpCode::OpLessEqual => {
                    let b = frame.slots.pop().unwrap();
                    let a = frame.slots.pop().unwrap();
                    frame
                        .slots
                        .push(if a <= b { Value::True } else { Value::False });
                }
                OpCode::OpNot => {
                    let value = frame.slots.pop().unwrap();
                    frame.slots.push(!value);
                }
                OpCode::OpTrue => frame.slots.push(Value::True),
                OpCode::OpFalse => frame.slots.push(Value::False),
                OpCode::OpNone => frame.slots.push(Value::None),
                OpCode::OpPrint => {
                    print_value(frame.slots.pop().unwrap());
                    println!();
                }
                OpCode::OpNegate => {
                    if !self.peek(&frame, 0).is_number() {
                        self.runtime_error(&mut frame, "Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let value = frame.slots.pop().unwrap();
                    frame.slots.push(-value);
                }
                OpCode::OpEof => {
                    return InterpretResult::Ok;
                }
                OpCode::OpEol => (),
                OpCode::OpSet => {
                    let slot = self.read_byte(&mut frame);
                    match slot {
                        OpCode::Number(slot) => {
                            if slot == usize::MAX {
                                self.runtime_error(&mut frame, &format!("Variable with this name already declared in the global scope.\nGlobal variables cannot be edited from a scope."));
                                return InterpretResult::RuntimeError;
                            }
                            frame.slots[slot as usize] = frame.slots.last().unwrap().clone();
                        }
                        _ => {
                            self.runtime_error(&mut frame, &format!("Unknown opcode {:?}", slot));
                            return InterpretResult::CompileError;
                        }
                    }
                }
                OpCode::OpGet => {
                    let slot = self.read_byte(&mut frame);
                    match slot {
                        OpCode::Number(slot) => {
                            if slot == usize::MAX {
                                self.runtime_error(&mut frame, &format!("Undefined variable."));
                                return InterpretResult::RuntimeError;
                            }
                            frame.slots.push(frame.slots[slot as usize].clone());
                        }
                        _ => {
                            self.runtime_error(&mut frame, &format!("Unknown opcode {:?}", slot));
                            return InterpretResult::CompileError;
                        }
                    }
                }
                OpCode::OpPop => {
                    frame.slots.pop();
                }
                OpCode::OpJumpIfTrue => {
                    let offset = self.read_byte(&mut frame).as_number();
                    if self.peek(&frame, 0).is_truthy() {
                        frame.ip += offset;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let offset = self.read_byte(&mut frame).as_number();
                    if !self.peek(&frame, 0).is_truthy() {
                        frame.ip += offset;
                    }
                }
                OpCode::OpJump => {
                    let offset = self.read_byte(&mut frame).as_number();
                    frame.ip += offset;
                }
                OpCode::OpLoop => {
                    let offset = self.read_byte(&mut frame).as_number();
                    frame.ip -= offset;
                }
                OpCode::OpCall => {
                    let arg_count = self.read_byte(&mut frame).as_number();
                    if !self.call_value(&mut frame, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                    frame = self.frames.last_mut().unwrap().clone();
                }
                _ => {
                    self.runtime_error(&mut frame, &format!("Unknown opcode {:?}", instruction));
                    return InterpretResult::CompileError;
                }
            }
        }
    }

    fn read_byte(&mut self, frame: &mut CallFrame) -> OpCode {
        if frame.ip >= frame.function.chunk.code.len() {
            todo!("Handle this error");
        }

        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_constant(&mut self, frame: &mut CallFrame) -> Value {
        let constant = self.read_byte(frame);
        match constant {
            OpCode::Number(index) => frame.function.chunk.constants[index].clone(),
            _ => panic!("Expected constant to be a number"),
        }
    }

    fn call_value(&mut self, frame: &mut CallFrame, arg_count: usize) -> bool {
        let value = self.peek(frame, arg_count);
        match value {
            Value::ObjFunction(function) => {
                self.call(frame, function);
                true
            }
            _ => {
                self.runtime_error(frame, "Can only call functions and classes.");
                false
            }
        }
    }

    fn call(&mut self, frame: &mut CallFrame, function: ObjFunction) {
        let arg_count = function.function_info.arg_names.len();
        let new_frame = CallFrame {
            ip: 0,
            function,
            slots: frame.slots.split_off(frame.slots.len() - arg_count),
        };
        self.frames.push(new_frame);
    }

    fn peek(&self, frame: &CallFrame, distance: usize) -> Value {
        frame.slots[frame.slots.len() - distance - 1].clone()
    }

    fn runtime_error(&self, frame: &mut CallFrame, message: &str) {
        eprintln!();
        eprintln!("{}", message);
        eprintln!(
            "[line {}] in script",
            frame.function.chunk.get_line(frame.ip - 1)
        );
    }
}
