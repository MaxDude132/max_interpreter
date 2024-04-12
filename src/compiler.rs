use crate::{
    chunk::{Chunk, OpCode},
    common::DEBUG_PRINT_CODE,
    object::ObjFunction,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};

use num_traits::FromPrimitive;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    next: Token,
    next_2: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            current: Token::new(TokenType::Empty, 0),
            previous: Token::new(TokenType::Empty, 0),
            next: Token::new(TokenType::Empty, 0),
            next_2: Token::new(TokenType::Empty, 0),
            had_error: false,
            panic_mode: false,
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.clone(), message);
    }

    fn error_at_previous(&mut self, message: &str) {
        self.error_at(&self.previous.clone(), message);
    }

    fn error_at_next(&mut self, message: &str) {
        self.error_at(&self.next.clone(), message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        if token.r#type == TokenType::Eof {
            eprintln!("[line {}] Error at end: {}", token.line, message);
        } else {
            eprintln!(
                "[line {}] Error at '{}': {}",
                token.line, token.lexeme, message
            );
        }
        self.had_error = true;
    }

    fn consume(&mut self, r#type: TokenType, message: &str) {
        if self.current.r#type == r#type || self.current.r#type == TokenType::Eof {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.next.clone();
        self.next = self.next_2.clone();

        loop {
            self.next_2 = self.scanner.scan_token();
            while self.current.r#type == TokenType::Empty {
                self.previous = self.current.clone();
                self.current = self.next.clone();
                self.next = self.next_2.clone();
                self.next_2 = self.scanner.scan_token();
            }
            if self.current.r#type != TokenType::Error {
                break;
            }

            self.error_at_next("Error at next token.");
        }
    }

    fn match_token(&mut self, r#type: TokenType) -> bool {
        if self.current.r#type != r#type {
            return false;
        }

        self.advance();
        return true;
    }

    // fn peek_previous(&self) -> Token {
    //     self.previous.clone()
    // }

    fn peek_current(&self) -> Token {
        self.current.clone()
    }

    fn peek_next(&self) -> Token {
        self.next.clone()
    }

    // fn peek_next_2(&self) -> Token {
    //     self.next_2.clone()
    // }

    fn check(&self, r#type: TokenType) -> bool {
        self.current.r#type == r#type
    }
}

static mut PARSER: Lazy<Parser> = Lazy::new(|| Parser::new(String::new()));

fn get_parser() -> &'static mut Parser {
    unsafe { &mut *PARSER }
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

struct ParseRule {
    precedence: Precedence,
    prefix: fn(&mut Compiler, bool),
    infix: fn(&mut Compiler, bool),
}

#[derive(Clone, Debug)]
pub struct Local {
    name: Token,
    depth: usize,
    type_: TokenType,
    is_initialized: bool,
}

#[derive(Clone, Debug)]
pub enum FunctionType {
    Function,
    Script,
    Method,
}

#[derive(Clone)]
pub struct Compiler {
    function: ObjFunction,
    function_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            function: ObjFunction::new(),
            function_type: FunctionType::Script,
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn immut_current_chunk(&self) -> &Chunk {
        &self.function.chunk
    }

    pub fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    pub fn compile(&mut self, source: String) -> ObjFunction {
        get_parser().scanner = Scanner::new(source);

        self.start_compiler();

        while !get_parser().match_token(TokenType::Eof) {
            self.declaration();
        }

        self.end_compiler();

        if get_parser().had_error {
            self.current_chunk().had_error = true;
        }
        return self.function.clone();
    }

    fn declaration(&mut self) {
        if get_parser().peek_current().r#type == TokenType::Identifier
            && (get_parser().peek_next().r#type == TokenType::Equal
                || get_parser().peek_next().r#type == TokenType::Newline)
            || get_parser().peek_current().r#type.is_type()
        {
            self.variable_assignment();
        } else if get_parser().peek_current().r#type == TokenType::Identifier
            && (get_parser().peek_next().r#type == TokenType::Colon
                || get_parser().peek_next().r#type == TokenType::LeftBrace)
        {
            self.function_declaration();
        } else {
            self.statement();
        }

        if get_parser().panic_mode {
            self.synchronize();
        }
    }

    fn function_declaration(&mut self) {
        let var_name_register =
            self.parse_variable("Expect function name.", TokenType::TypeFunction);
        self.locals[var_name_register.as_number()].is_initialized = true;
        self.function(FunctionType::Function);
        self.set_variable(var_name_register);
    }

    fn function(&mut self, function_type: FunctionType) {
        let mut compiler = Compiler::new();
        compiler.function_type = function_type;
        compiler.function.name = get_parser().previous.lexeme.clone();
        compiler.begin_scope();

        if get_parser().peek_current().r#type == TokenType::Colon {
            get_parser().advance();
            loop {
                if !get_parser().peek_current().r#type.is_type() {
                    get_parser().error_at_current("Expect variable type annotation.");
                } else if get_parser().peek_next().r#type != TokenType::Identifier {
                    get_parser().error_at_next("Expect variable name.");
                }
                compiler.variable_assignment();
                if !get_parser().match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        get_parser().consume(TokenType::LeftBrace, "Expect '{' before function body.");
        compiler.block();

        let func = compiler.end_compiler();
        let byte_2 = self.make_constant(Value::ObjFunction(func));
        self.emit_2_bytes(OpCode::OpConstant, byte_2);
    }

    fn variable_assignment(&mut self) {
        let mut var_type = TokenType::None;
        if get_parser().peek_current().r#type.is_type() {
            var_type = get_parser().current.r#type;
            get_parser().advance();
        }

        let var_name_register = self.parse_variable("Expect variable name.", var_type);

        println!(
            "{:?} - {:?} - {:?}",
            var_name_register,
            var_type,
            get_parser().previous.lexeme
        );
        if get_parser().match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_constant(var_type.get_none_type());
        }

        self.set_variable(var_name_register);
        self.locals[var_name_register.as_number()].is_initialized = true;
    }

    fn parse_variable(&mut self, message: &str, var_type: TokenType) -> OpCode {
        get_parser().consume(TokenType::Identifier, message);

        let index = self.declare_variable(var_type);
        return OpCode::Number(index);
    }

    fn declare_variable(&mut self, var_type: TokenType) -> usize {
        let name = get_parser().previous.clone();
        return self.add_local(name, var_type);
    }

    fn add_local(&mut self, name: Token, var_type: TokenType) -> usize {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].depth == 0
                && self.scope_depth != 0
                && name.lexeme == self.locals[i].name.lexeme
            {
                get_parser()
                    .error_at_current(&format!("Variable with name {} already declared in the global scope.\nGlobal variables cannot be edited from a scope.", name.lexeme));
                return usize::MAX;
            } else if name.lexeme == self.locals[i].name.lexeme {
                return i;
            }
        }

        let local = Local {
            name,
            depth: self.scope_depth,
            type_: var_type,
            is_initialized: false,
        };
        self.locals.push(local);
        return self.locals.len() - 1;
    }

    fn set_variable(&mut self, var_name_register: OpCode) {
        let value;
        match self.immut_current_chunk().constants.last() {
            None => {
                get_parser().error_at_previous("No value found to assign to the variable.");
                return;
            }
            Some(v) => {
                value = v;
            }
        }
        let local = self.locals[var_name_register.as_number()].clone();

        if !local.type_.is_correct_type(value) {
            get_parser().error_at_previous(&format!(
                "Variable {} is of type {} but value is of type {}",
                local.name.lexeme,
                local.type_,
                value.type_of()
            ));
        }
        self.emit_2_bytes(OpCode::OpSet, var_name_register);
    }

    fn synchronize(&mut self) {
        get_parser().panic_mode = false;

        while get_parser().current.r#type != TokenType::Eof {
            if get_parser().previous.r#type == TokenType::Newline {
                return;
            }

            get_parser().advance();
        }
    }

    fn statement(&mut self) {
        if get_parser().match_token(TokenType::Print) {
            self.print_statement();
        } else if get_parser().match_token(TokenType::If) {
            self.if_statement();
        } else if get_parser().match_token(TokenType::While) {
            self.while_statement();
        } else if get_parser().match_token(TokenType::For) {
            self.for_statement();
        } else if get_parser().match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn for_statement(&mut self) {
        todo!("Finish for loops when methods are implemented.");
        // self.begin_scope();
        // let loop_start = self.current_chunk().code.len();

        // println!("{:?}", get_parser().peek_next_2());
        // // self.variable_assignment();
        // get_parser().consume(
        //     TokenType::In,
        //     "Expect in after variable declaration in for loop.",
        // );

        // self.statement();
        // self.emit_loop(loop_start);
        // self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();
        self.expression();

        let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::OpPop);

        // Handle break statement
        if get_parser().match_token(TokenType::Break) {
            self.emit_jump(OpCode::OpJump);
        }
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::OpLoop);
        let offset = self.current_chunk().code.len() - loop_start + 2;
        self.emit_byte(OpCode::Number(offset));
    }

    fn if_statement(&mut self) {
        self.expression();

        let then_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::OpJump);

        self.patch_jump(then_jump);

        if get_parser().match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::OpPop);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(OpCode::Number(0));
        return self.current_chunk().code.len() - 1;
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().code.len() - offset - 1;
        self.current_chunk().code[offset] = OpCode::Number(jump);
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn block(&mut self) {
        while !get_parser().check(TokenType::RightBrace) && !get_parser().check(TokenType::Eof) {
            self.declaration();
        }

        get_parser().consume(TokenType::RightBrace, "Expect '}' after block")
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        for i in (0..self.locals.len()).rev() {
            if self.locals[i].depth > self.scope_depth {
                self.emit_byte(OpCode::OpPop);
                self.locals.pop();
            }
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.emit_eol();
    }

    fn expression(&mut self) {
        self.parse_precendence(Precedence::Assignment);
    }

    fn print_statement(&mut self) {
        self.expression();
        get_parser().consume(TokenType::Newline, "Expect newline after value.");
        self.emit_byte(OpCode::OpPrint);
        self.emit_eol();
    }

    fn parse_precendence(&mut self, precedence: Precedence) {
        get_parser().advance();
        let prefix_rule = self.get_rule(get_parser().previous.r#type).prefix;
        if prefix_rule == Compiler::none
            && get_parser().previous.r#type != TokenType::Newline
            && get_parser().current.r#type == TokenType::Newline
        {
            get_parser().error_at_previous("Expect expression.");
            return;
        }

        let can_assign = precedence as u8 <= Precedence::Assignment as u8;
        prefix_rule(self, can_assign);

        while precedence as u8 <= self.get_rule(get_parser().current.r#type).precedence as u8 {
            get_parser().advance();
            let infix_rule = self.get_rule(get_parser().previous.r#type).infix;
            infix_rule(self, can_assign);
        }

        if can_assign && get_parser().match_token(TokenType::Equal) {
            get_parser().error_at_previous("Invalid assignment target.");
        }
    }

    fn integer(&mut self, _can_assign: bool) {
        let value = get_parser().previous.lexeme.parse::<i64>().unwrap();
        self.emit_constant(Value::Integer(value));
    }

    fn float(&mut self, _can_assign: bool) {
        let value = get_parser().previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Float(value));
    }

    fn string(&mut self, _can_assign: bool) {
        let value = get_parser().previous.lexeme.parse::<String>().unwrap();
        self.emit_constant(Value::String(value));
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        get_parser().consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = get_parser().previous.r#type;
        self.parse_precendence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::OpNegate),
            TokenType::Bang => self.emit_byte(OpCode::OpNot),
            _ => panic!("Invalid unary type."),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = get_parser().previous.r#type;
        let rule = self.get_rule(operator_type);
        let precedence = FromPrimitive::from_u8(rule.precedence as u8 + 1).unwrap();
        self.parse_precendence(precedence);

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::OpAdd),
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract),
            TokenType::Star => self.emit_byte(OpCode::OpMultiply),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide),
            TokenType::BangEqual => self.emit_byte(OpCode::OpNotEqual),
            TokenType::EqualEqual => self.emit_byte(OpCode::OpEqual),
            TokenType::Greater => self.emit_byte(OpCode::OpGreater),
            TokenType::GreaterEqual => self.emit_byte(OpCode::OpGreaterEqual),
            TokenType::Less => self.emit_byte(OpCode::OpLess),
            TokenType::LessEqual => self.emit_byte(OpCode::OpLessEqual),
            _ => panic!("Invalid binary type."),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match get_parser().previous.r#type {
            TokenType::True => self.emit_byte(OpCode::OpTrue),
            TokenType::False => self.emit_byte(OpCode::OpFalse),
            TokenType::None => self.emit_byte(OpCode::OpNone),
            _ => panic!("Invalid literal type."),
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(get_parser().previous.lexeme.clone(), can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        let arg = self.resolve_local(&name);

        if can_assign && get_parser().match_token(TokenType::Equal) {
            self.expression();
            self.set_variable(arg);
        }
        self.emit_2_bytes(OpCode::OpGet, arg);
    }

    fn resolve_local(&mut self, name: &String) -> OpCode {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name.lexeme == *name {
                if !self.locals[i].is_initialized {
                    get_parser().error_at_previous(&format!(
                        "Variable {} is used before being initialized.",
                        name
                    ));
                }
                return OpCode::Number(i);
            }
        }

        get_parser().error_at_previous(&format!("Variable {} could not be found.", name));

        return OpCode::Number(usize::MAX);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::OpJumpIfFalse);

        self.emit_byte(OpCode::OpPop);
        self.parse_precendence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::OpJumpIfTrue);

        self.emit_byte(OpCode::OpPop);
        self.parse_precendence(Precedence::Or);

        self.patch_jump(end_jump);
    }

    fn none(&mut self, _can_assign: bool) {}

    fn get_rule(&self, r#type: TokenType) -> ParseRule {
        match r#type {
            TokenType::Float => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::float,
                infix: Compiler::none,
            },
            TokenType::Integer => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::integer,
                infix: Compiler::none,
            },
            TokenType::String => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::string,
                infix: Compiler::none,
            },
            TokenType::True => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::literal,
                infix: Compiler::none,
            },
            TokenType::False => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::literal,
                infix: Compiler::none,
            },
            TokenType::None => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::literal,
                infix: Compiler::none,
            },
            TokenType::LeftParen => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::grouping,
                infix: Compiler::none,
            },
            TokenType::Minus => ParseRule {
                precedence: Precedence::Term,
                prefix: Compiler::unary,
                infix: Compiler::binary,
            },
            TokenType::Plus => ParseRule {
                precedence: Precedence::Term,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Star => ParseRule {
                precedence: Precedence::Factor,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Slash => ParseRule {
                precedence: Precedence::Factor,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::And => ParseRule {
                precedence: Precedence::And,
                prefix: Compiler::none,
                infix: Compiler::and,
            },
            TokenType::Or => ParseRule {
                precedence: Precedence::Or,
                prefix: Compiler::none,
                infix: Compiler::or,
            },
            TokenType::EqualEqual => ParseRule {
                precedence: Precedence::Equality,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Greater => ParseRule {
                precedence: Precedence::Equality,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::GreaterEqual => ParseRule {
                precedence: Precedence::Equality,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Less => ParseRule {
                precedence: Precedence::Equality,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::LessEqual => ParseRule {
                precedence: Precedence::Equality,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Bang => ParseRule {
                precedence: Precedence::Unary,
                prefix: Compiler::unary,
                infix: Compiler::none,
            },
            TokenType::BangEqual => ParseRule {
                precedence: Precedence::Unary,
                prefix: Compiler::none,
                infix: Compiler::binary,
            },
            TokenType::Identifier => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::variable,
                infix: Compiler::none,
            },
            _ => ParseRule {
                precedence: Precedence::None,
                prefix: Compiler::none,
                infix: Compiler::none,
            },
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_2_bytes(OpCode::OpConstant, constant)
    }

    fn make_constant(&mut self, value: Value) -> OpCode {
        let chunk = self.current_chunk();
        let constant = chunk.add_constant(value);
        OpCode::Number(constant)
    }

    fn emit_eof(&mut self) {
        self.emit_byte(OpCode::OpEof);
    }

    fn emit_eol(&mut self) {
        self.emit_byte(OpCode::OpEol);
    }

    fn start_compiler(&mut self) {
        get_parser().advance();
    }

    fn end_compiler(&mut self) -> ObjFunction {
        self.emit_eof();
        if DEBUG_PRINT_CODE && !self.current_chunk().had_error {
            let func_name = format!("{}", &self.function);
            self.immut_current_chunk()
                .disassemble(if self.function.name == "" {
                    "<script>"
                } else {
                    &func_name
                });
        }
        return self.function.clone();
    }

    fn emit_byte(&mut self, byte: OpCode) {
        let line = get_parser().previous.line;
        self.current_chunk().write(byte, line);
    }

    fn emit_2_bytes(&mut self, byte1: OpCode, byte2: OpCode) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }
}
