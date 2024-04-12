use std::fmt::Display;

use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct ObjFunction {
    pub arity: u8,
    pub name: String,
    pub chunk: Chunk,
}

impl ObjFunction {
    pub fn new() -> ObjFunction {
        ObjFunction {
            arity: 0,
            name: String::new(),
            chunk: Chunk::new(),
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
