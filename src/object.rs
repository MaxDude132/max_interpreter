use std::fmt::Display;

use crate::{chunk::Chunk, scanner::TokenType};

#[derive(Debug, Clone)]
pub struct ObjFunction {
    pub name: String,
    pub chunk: Chunk,
    pub function_info: FunctionInfo,
    pub functions_count: usize,
}

impl ObjFunction {
    pub fn new() -> ObjFunction {
        ObjFunction {
            name: String::new(),
            chunk: Chunk::new(),
            function_info: FunctionInfo::new(String::new()),
            functions_count: 0,
        }
    }

    pub fn had_error(&self) -> bool {
        self.chunk.had_error
    }
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = if !self.name.is_empty() {
            &self.name
        } else {
            "<script>"
        };
        write!(f, "<function {}>", name)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<TokenType>,
}

impl FunctionInfo {
    pub fn new(name: String) -> FunctionInfo {
        FunctionInfo {
            name,
            arg_names: Vec::new(),
            arg_types: Vec::new(),
        }
    }
}
