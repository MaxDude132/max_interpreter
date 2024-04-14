use crate::common::DEBUG_TRACE_EXECUTION;
use crate::compiler::{Compiler, FunctionType};
use crate::object::ObjFunction;
use crate::{
    chunk::OpCode,
    value::{print_value, Value},
};

macro_rules! binary_op {
    ($vm:expr, $operator:tt) => {
        {
            let b = $vm.current_frame().slots.pop().unwrap();
            let a = $vm.current_frame().slots.pop().unwrap();
            let val = a $operator b;
            match val {
                Ok(val) => $vm.current_frame().slots.push(val),
                Err(message) => {
                    $vm.runtime_error(&message);
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

#[derive(Clone, Debug)]
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

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.read_byte();
            if DEBUG_TRACE_EXECUTION {
                let frame = self.current_frame();
                frame
                    .function
                    .chunk
                    .disassemble_instruction(&instruction, frame.ip - 1);
            }

            match instruction {
                OpCode::OpConstant => {
                    let constant = self.read_constant();
                    self.current_frame().slots.push(constant);
                }
                OpCode::OpAdd => binary_op!(self, +),
                OpCode::OpSubtract => binary_op!(self, -),
                OpCode::OpMultiply => binary_op!(self, *),
                OpCode::OpDivide => binary_op!(self, /),
                OpCode::OpEqual => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(if a == b {
                        Value::True
                    } else {
                        Value::False
                    });
                }
                OpCode::OpNotEqual => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(if a != b {
                        Value::True
                    } else {
                        Value::False
                    });
                }
                OpCode::OpGreater => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame()
                        .slots
                        .push(if a > b { Value::True } else { Value::False });
                }
                OpCode::OpGreaterEqual => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(if a >= b {
                        Value::True
                    } else {
                        Value::False
                    });
                }
                OpCode::OpLess => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame()
                        .slots
                        .push(if a < b { Value::True } else { Value::False });
                }
                OpCode::OpLessEqual => {
                    let b = self.current_frame().slots.pop().unwrap();
                    let a = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(if a <= b {
                        Value::True
                    } else {
                        Value::False
                    });
                }
                OpCode::OpNot => {
                    let value = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(!value);
                }
                OpCode::OpTrue => self.current_frame().slots.push(Value::True),
                OpCode::OpFalse => self.current_frame().slots.push(Value::False),
                OpCode::OpNone => self.current_frame().slots.push(Value::None),
                OpCode::OpPrint => {
                    print_value(self.current_frame().slots.pop().unwrap());
                    println!();
                }
                OpCode::OpNegate => {
                    if !self.peek(0).is_number() {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let value = self.current_frame().slots.pop().unwrap();
                    self.current_frame().slots.push(-value);
                }
                OpCode::OpEof => {
                    return InterpretResult::Ok;
                }
                OpCode::OpEol => (),
                OpCode::OpSet => {
                    let slot = self.read_byte();
                    match slot {
                        OpCode::Number(slot) => {
                            if slot == usize::MAX {
                                self.runtime_error( &format!("Variable with this name already declared in the global scope.\nGlobal variables cannot be edited from a scope."));
                                return InterpretResult::RuntimeError;
                            }
                            self.current_frame().slots[slot as usize] =
                                self.current_frame().slots.last().unwrap().clone();
                        }
                        _ => {
                            self.runtime_error(&format!("Unknown opcode {:?}", slot));
                            return InterpretResult::CompileError;
                        }
                    }
                }
                OpCode::OpGet => {
                    let slot = self.read_byte();
                    match slot {
                        OpCode::Number(slot) => {
                            if slot == usize::MAX {
                                self.runtime_error(&format!("Undefined variable."));
                                return InterpretResult::RuntimeError;
                            }
                            let frame = self.current_frame();
                            frame.slots.push(frame.slots[slot as usize].clone());
                        }
                        _ => {
                            self.runtime_error(&format!("Unknown opcode {:?}", slot));
                            return InterpretResult::CompileError;
                        }
                    }
                }
                OpCode::OpPop => {
                    self.current_frame().slots.pop();
                }
                OpCode::OpJumpIfTrue => {
                    let offset = self.read_byte().as_number();
                    if self.peek(0).is_truthy() {
                        self.current_frame().ip += offset;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let offset = self.read_byte().as_number();
                    if !self.peek(0).is_truthy() {
                        self.current_frame().ip += offset;
                    }
                }
                OpCode::OpJump => {
                    let offset = self.read_byte().as_number();
                    self.current_frame().ip += offset;
                }
                OpCode::OpLoop => {
                    let offset = self.read_byte().as_number();
                    self.current_frame().ip -= offset;
                }
                OpCode::OpCall => {
                    let arg_count = self.read_byte().as_number();
                    if !self.call_value(arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpReturn => {
                    let result = self.current_frame().slots.pop().unwrap();
                    self.frames.pop();
                    if self.frames.is_empty() {
                        return InterpretResult::Ok;
                    }
                    self.current_frame().slots.push(result);
                }
                _ => {
                    self.runtime_error(&format!("Unknown opcode {:?}", instruction));
                    return InterpretResult::CompileError;
                }
            }
        }
    }

    fn read_byte(&mut self) -> OpCode {
        let frame = self.current_frame();

        if frame.ip >= frame.function.chunk.code.len() {
            todo!("Handle this error");
        }

        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.read_byte();
        match constant {
            OpCode::Number(index) => self.current_frame().function.chunk.constants[index].clone(),
            _ => panic!("Expected constant to be a number"),
        }
    }

    fn call_value(&mut self, arg_count: usize) -> bool {
        let value = self.peek(arg_count);
        match value {
            Value::ObjFunction(function) => {
                self.call(function);
                true
            }
            _ => {
                self.runtime_error(&format!(
                    "Can only call functions and classes. Got {:?} instead.",
                    value
                ));
                false
            }
        }
    }

    fn call(&mut self, function: ObjFunction) {
        let frame = self.current_frame();

        let arg_count = function.function_info.arg_names.len();
        let at = frame.slots.len() - arg_count;

        let mut new_slots = frame.slots[0..frame.function.functions_count].to_vec();
        new_slots.extend(frame.slots.split_off(at));

        let new_frame = CallFrame {
            ip: 0,
            function,
            slots: new_slots,
        };
        self.frames.push(new_frame);
    }

    fn peek(&mut self, distance: usize) -> Value {
        let frame = self.current_frame();
        frame.slots[frame.slots.len() - distance - 1].clone()
    }

    fn runtime_error(&mut self, message: &str) {
        let frame = self.current_frame();

        eprintln!();
        eprintln!("{}", message);
        eprintln!(
            "[line {}] in script",
            frame.function.chunk.get_line(frame.ip - 1)
        );

        // for i in (0..self.frames.len()).rev() {
        //     let frame = &self.frames[i];
        //     let line = frame.function.chunk.get_line(frame.ip);
        //     eprint!("[line {}] in ", line);
        //     if !frame.function.name.is_empty() {
        //         eprint!("function {}", frame.function.name);
        //     } else {
        //         eprint!("script");
        //     }
        //     eprintln!();
        // }
    }
}
