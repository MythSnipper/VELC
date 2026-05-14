#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::compiler::lexer::*;

#[derive(Default, Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>
}
impl Program {
    pub fn new() -> Self {
        Self { 
            items: Vec::new()
        }
    }
}

//Top level
#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(FunctionDecl),
    GlobalVar(VarDecl),
    Assembly(AssemblyDecl),
}
#[derive(Debug, Clone)]
pub struct Param {
    pub Type: TypeName,
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub ret: TypeName,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Stmt,
}

#[derive(Debug, Clone)]
pub struct AsmInput {
    pub name: String,
    pub reg: String,
}

#[derive(Debug, Clone)]
pub struct AsmOutput {
    pub reg: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AssemblyDecl {
    pub code: String,
    pub section: String,
    pub inputs: Vec<AsmInput>,
    pub outputs: Vec<AsmOutput>,
}



//Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Char,
    Float32,
    Float64,
    String,
    Void,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    Builtin(BuiltinType),

    Pointer(Box<TypeName>),
    Array(Box<TypeName>, usize),



    Function{
        ret: Box<TypeName>,
        params: Vec<TypeName>,
    },
    IntLiteral,
    FloatLiteral,
}


//Operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixOp {
    LogicalNot,
    BitwiseNot,
    Inc,
    Dec,
    Plus,
    Neg,
    Ref,
    Deref,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostfixOp {
    Inc,
    Dec,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Lshift,
    Rshift,

    LogicalAnd,
    LogicalOr,
    LogicalXor,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LshiftAssign,
    RshiftAssign,
    NotAssign,
}

//Statements
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub Type: TypeName,
    pub name: String,
    pub init: Option<Expr>,
}
#[derive(Debug, Clone)]
pub enum ForInit {
    VarDecl(VarDecl),
    Expr(Expr),
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    VarDecl(VarDecl),

    Return(Option<Expr>),
    Break,
    Continue,

    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    While {
        cond: Expr,
        body: Box<Stmt>,
    },

    For {
        init: Option<ForInit>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Box<Stmt>,
    },

    Expr(Expr),

    Assembly(AssemblyDecl),
    Empty,
}

//Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    BoolLiteral(bool),
    IntLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),

    Identifier(String),

    Grouping(Box<Expr>),

    Prefix {
        op: PrefixOp,
        expr: Box<Expr>,
    },

    Postfix {
        op: PostfixOp,
        expr: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    Assign {
        left: Box<Expr>,
        op: AssignOp,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },

    Member {
        base: Box<Expr>,
        field: String,
    },

    Cast {
        Type: TypeName,
        expr: Box<Expr>,
    },
}

#[derive(Default, Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}
impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            index: 0,
        }
    }
    pub fn from_tokens(t: &Vec<Token>) -> Self {
        Self {
            tokens: t.clone(),
            index: 0,
        }
    }

    pub fn run(&mut self) -> Result<Program, String> {
        self.parse_program()
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.index += 1;
        }
        self.prev()
    }
    fn prev(&self) -> &Token {
        if let Some(token) = self.tokens.get(self.index-1) {
            token
        }
        else {
            &Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan{row: 0, col: 0}}
        }
    }
    fn next(&self) -> &Token {
        if let Some(token) = self.tokens.get(self.index+1) {
            token
        }
        else {
            &Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan{row: 0, col: 0}}
        }
    }
    fn next_next(&self) -> &Token {
        if let Some(token) = self.tokens.get(self.index+2) {
            token
        }
        else {
            &Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan{row: 0, col: 0}}
        }
    }
    fn get(&self) -> &Token {
        self.tokens.get(self.index).unwrap()
    }
    fn is_at_end(&self) -> bool {
        self.get().Type == TokenType::EOF
    }
    fn check(&self, expected: &TokenType) -> bool {
        self.get().Type == *expected
    }
    fn matches(&mut self, expected: &TokenType) -> bool {
        if self.check(expected) {
            self.advance();
            true
        }
        else {
            false
        }
    }
    fn expect(&mut self, expected: &TokenType) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Parser:expect";
        if self.check(expected) {
            Ok(self.advance().clone())
        }
        else {
            self.error(ERROR_STRING_ROOT, &format!("Expected {:?} token", expected))
        }
    }
    fn is_type_keyword(&self) -> bool {
        matches!(
            self.get().Type,
            TokenType::Bool_Keyword
                | TokenType::Int8_Keyword
                | TokenType::Int16_Keyword
                | TokenType::Int32_Keyword
                | TokenType::Int64_Keyword
                | TokenType::Uint8_Keyword
                | TokenType::Uint16_Keyword
                | TokenType::Uint32_Keyword
                | TokenType::Uint64_Keyword
                | TokenType::Char_Keyword
                | TokenType::Float32_Keyword
                | TokenType::Float64_Keyword
                | TokenType::String_Keyword
                | TokenType::Void_Keyword
        )
    }
    fn parse_type(&mut self) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_type";
        let mut ty = match self.get().Type {
            TokenType::Bool_Keyword => TypeName::Builtin(BuiltinType::Bool),
            TokenType::Int8_Keyword => TypeName::Builtin(BuiltinType::Int8),
            TokenType::Int16_Keyword => TypeName::Builtin(BuiltinType::Int16),
            TokenType::Int32_Keyword => TypeName::Builtin(BuiltinType::Int32),
            TokenType::Int64_Keyword => TypeName::Builtin(BuiltinType::Int64),
            TokenType::Uint8_Keyword => TypeName::Builtin(BuiltinType::Uint8),
            TokenType::Uint16_Keyword => TypeName::Builtin(BuiltinType::Uint16),
            TokenType::Uint32_Keyword => TypeName::Builtin(BuiltinType::Uint32),
            TokenType::Uint64_Keyword => TypeName::Builtin(BuiltinType::Uint64),
            TokenType::Char_Keyword => TypeName::Builtin(BuiltinType::Char),
            TokenType::Float32_Keyword => TypeName::Builtin(BuiltinType::Float32),
            TokenType::Float64_Keyword => TypeName::Builtin(BuiltinType::Float64),
            TokenType::String_Keyword => TypeName::Builtin(BuiltinType::String),
            TokenType::Void_Keyword => TypeName::Builtin(BuiltinType::Void),
            _ => return self.error(ERROR_STRING_ROOT, "Expected type keyword")
        };
        self.advance();
    
        //pointers
        while self.matches(&TokenType::Asterisk) {
            ty = TypeName::Pointer(Box::new(ty));
        }
    
        Ok(ty)
    }
    fn consume_identifier(&mut self) -> Result<String, String> {
        let ERROR_STRING_ROOT = "velc:Parser:consume_identifier";
        if self.get().Type != TokenType::Identifier {
            return self.error(ERROR_STRING_ROOT, "Current token is not an identifier");
        }
    
        match &self.get().Value {
            TokenValue::String(s) => {
                let out = s.clone();
                self.advance();
                Ok(out)
            }
            _ => self.error(ERROR_STRING_ROOT, "Identifier token missing string value")
        }
    }
    fn is_cast(&self) -> bool {
        if self.get().Type != TokenType::Lparen {
            return false;
        }
        let mut i = self.index + 1;

        let is_type_keyword = |t: &TokenType| {
            matches!(
                t,
                TokenType::Bool_Keyword
                    | TokenType::Int8_Keyword
                    | TokenType::Int16_Keyword
                    | TokenType::Int32_Keyword
                    | TokenType::Int64_Keyword
                    | TokenType::Uint8_Keyword
                    | TokenType::Uint16_Keyword
                    | TokenType::Uint32_Keyword
                    | TokenType::Uint64_Keyword
                    | TokenType::Char_Keyword
                    | TokenType::Float32_Keyword
                    | TokenType::Float64_Keyword
                    | TokenType::String_Keyword
                    | TokenType::Void_Keyword
            )
        };

        let Some(token) = self.tokens.get(i) else {
            return false;
        };

        if !is_type_keyword(&token.Type) {
            return false;
        }

        i += 1;

        while let Some(token) = self.tokens.get(i) {
            if token.Type == TokenType::Asterisk {
                i += 1;
            }
            else {
                break;
            }
        }

        match self.tokens.get(i) {
            Some(token) if token.Type == TokenType::Rparen => true,
            _ => false,
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut program = Program::new();
        while !self.is_at_end() {
            let top_level = self.parse_top_level()?;
            program.items.push(top_level);
        }
        Ok(program)
    }
    fn parse_top_level(&mut self) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_top_level";
    
        if self.check(&TokenType::Assembly) {
            return self.parse_top_level_assembly();
        }
    
        if self.is_type_keyword() {
            let base = self.parse_type()?;
    
            // Function definition:
            // int32 foo(...)
            if self.check(&TokenType::Identifier) && self.next().Type == TokenType::Lparen {
                let id = self.consume_identifier()?;
                return self.parse_function(base, id);
            }
    
            // Otherwise treat as variable declarator
            let (id, Type) = self.parse_decl_name_and_type(base)?;
            return self.parse_global_var_decl(Type, id);
        }
    
        self.error(ERROR_STRING_ROOT, "Expected type keyword")
    }
    fn parse_top_level_assembly(&mut self) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_top_level_assembly";
    
        match &self.get().Value {
            TokenValue::String(text) => {
                let decl = self.parse_asm_decl(text)?;
                self.advance();
                Ok(TopLevel::Assembly(decl))
            }
            _ => self.error(ERROR_STRING_ROOT, "Assembly token missing string value")
        }
    }
    fn parse_function(&mut self, t: TypeName, id: String) -> Result<TopLevel, String> {
        
        self.expect(&TokenType::Lparen)?;

        let mut params = Vec::new();

        if !self.check(&TokenType::Rparen) { //if there are arguments
            loop {
                let param_type = self.parse_type()?;
                let param_name = self.consume_identifier()?;
                params.push(Param {
                    Type: param_type,
                    name: param_name,
                });

                if self.matches(&TokenType::Comma) {
                    continue;
                }
                else {
                    break;
                }
            }
        }

        self.expect(&TokenType::Rparen)?;

        let body = self.parse_stmt()?;

        Ok(TopLevel::Function(FunctionDecl {
            ret: t,
            name: id,
            params: params,
            body: body,
        }))


    }
    fn parse_global_var_decl(&mut self, t: TypeName, id: String) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_global_var_decl";
        
        let init = if self.matches(&TokenType::Assign) {
            if matches!(t, TypeName::Array(_, _)) {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Global array initializers are not implemented yet",
                );
            }

            Some(self.parse_expr()?)
        }
        else {
            None
        };
    
        self.expect(&TokenType::Semicolon)?;
        Ok(TopLevel::GlobalVar(VarDecl {
            Type: t,
            name: id,
            init: init,
        }))
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_block";
        
        let mut stmts: Vec<Stmt> = Vec::new();

        self.expect(&TokenType::Lbrace)?;

        while !self.check(&TokenType::Rbrace) {
            if self.is_at_end() {
                return self.error(ERROR_STRING_ROOT, "expected }");
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenType::Rbrace)?;

        Ok(Stmt::Block(stmts))
    }
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_stmt";

        if self.is_at_end() {
            return self.error(ERROR_STRING_ROOT, "Unexpected EOF");
        }

        let stmt = if self.is_type_keyword() {
            //VarDecl
            self.parse_var_decl_stmt()?
        }
        else if self.check(&TokenType::Lbrace) {
            //Block
            self.parse_block()?
        }
        else {
            match self.get().Type {
                TokenType::Semicolon => {
                    self.advance();
                    Stmt::Empty
                }
                TokenType::If_Keyword => {
                    self.parse_if_stmt()?
                }
                TokenType::Else_Keyword => {
                    return self.error(ERROR_STRING_ROOT, "Unexpected else keyword");
                }
                TokenType::While_Keyword => {
                    self.parse_while_stmt()?
                }
                TokenType::For_Keyword => {
                    self.parse_for_stmt()?
                }
                TokenType::Return_Keyword => {
                    self.parse_return_stmt()?
                }
                TokenType::Break_Keyword => {
                    self.advance();
                    self.expect(&TokenType::Semicolon)?;
                    Stmt::Break
                }
                TokenType::Continue_Keyword => {
                    self.advance();
                    self.expect(&TokenType::Semicolon)?;
                    Stmt::Continue
                }
                TokenType::Assembly => {
                    self.parse_asm_stmt()?
                }
                _ => {
                    self.parse_expr_stmt()?
                }
            }
        };

        Ok(stmt)
    }
    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {

        self.expect(&TokenType::Return_Keyword)?;

        if self.matches(&TokenType::Semicolon) {
            return Ok(Stmt::Return(None))
        }
        
        let expr = self.parse_expr()?;

        self.expect(&TokenType::Semicolon)?;

        Ok(Stmt::Return(Some(expr)))
    }
    
    fn parse_decl_name_and_type(&mut self, base: TypeName) -> Result<(String, TypeName), String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_decl_name_and_type";
    
        // Simple case:
        // int32 x;
        // or array decl
        // int32 x[3][4];
        if self.check(&TokenType::Identifier) {
            let name = self.consume_identifier()?;
            let full_type = self.parse_array_suffixes(base)?;
            return Ok((name, full_type));
        }
    
        // Function pointer case:
        // int32 (*ptr)(int32, int32);
        if self.matches(&TokenType::Lparen) {
            self.expect(&TokenType::Asterisk)?;
            let name = self.consume_identifier()?;
            self.expect(&TokenType::Rparen)?;
    
            self.expect(&TokenType::Lparen)?;
    
            let mut params = Vec::new();
    
            if !self.check(&TokenType::Rparen) {
                loop {
                    let param_type = self.parse_type()?;
                    params.push(param_type);
    
                    // Optional parameter name, ignored for function pointer types
                    if self.check(&TokenType::Identifier) {
                        self.consume_identifier()?;
                    }
    
                    if self.matches(&TokenType::Comma) {
                        continue;
                    }
                    else {
                        break;
                    }
                }
            }
    
            self.expect(&TokenType::Rparen)?;
    
            return Ok((
                name,
                TypeName::Pointer(Box::new(TypeName::Function {
                    ret: Box::new(base),
                    params,
                })),
            ));
        }
    
        self.error(ERROR_STRING_ROOT, "Expected identifier or function pointer declarator")
    }
    fn parse_var_decl_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_var_decl_stmt";

        let base = self.parse_type()?;
        let (id, t) = self.parse_decl_name_and_type(base)?;
    
        let init = if self.matches(&TokenType::Assign) {
            if matches!(t, TypeName::Array(_, _)) {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Array initializers not implemented"
                );
            }

            Some(self.parse_expr()?)
        }
        else {
            None
        };
    
        self.expect(&TokenType::Semicolon)?;
    
        Ok(Stmt::VarDecl(VarDecl {
            Type: t,
            name: id,
            init,
        }))
    }
    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;

        self.expect(&TokenType::Semicolon)?;

        Ok(Stmt::Expr(expr))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {

        self.expect(&TokenType::If_Keyword)?;

        self.expect(&TokenType::Lparen)?;

        let cond = self.parse_expr()?;

        self.expect(&TokenType::Rparen)?;

        let then_br = Box::new(self.parse_stmt()?);

        let else_br = if self.matches(&TokenType::Else_Keyword) {
            Some(Box::new(self.parse_stmt()?))
        }
        else {
            None
        };

        Ok(Stmt::If{
            cond: cond,
            then_branch: then_br,
            else_branch: else_br
        })
    }
    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {

        self.expect(&TokenType::While_Keyword)?;

        self.expect(&TokenType::Lparen)?;

        let cond = self.parse_expr()?;

        self.expect(&TokenType::Rparen)?;

        let body = Box::new(self.parse_stmt()?);

        Ok(Stmt::While{
            cond: cond,
            body: body
        })

    }
    fn parse_for_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_for_stmt";
        self.expect(&TokenType::For_Keyword)?;

        self.expect(&TokenType::Lparen)?;

        let init = if self.check(&TokenType::Semicolon) {
            None
        }
        else if self.is_type_keyword() {
            let base = self.parse_type()?;
            let (name, full_type) = self.parse_decl_name_and_type(base)?;

            let init = if self.matches(&TokenType::Assign) {
                if matches!(full_type, TypeName::Array(_, _)) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Array initializers are not implemented yet",
                    );
                }
                Some(self.parse_expr()?)
            }
            else {
                None
            };

            Some(ForInit::VarDecl(VarDecl {
                Type: full_type,
                name: name,
                init: init
            }))
        }
        else {
            Some(ForInit::Expr(self.parse_expr()?))
        };

        self.expect(&TokenType::Semicolon)?;

        let cond = if self.check(&TokenType::Semicolon) {
            None
        }
        else {
            Some(self.parse_expr()?)
        };

        self.expect(&TokenType::Semicolon)?;

        let step = if self.check(&TokenType::Rparen) {
            None
        }
        else {
            Some(self.parse_expr()?)
        };

        self.expect(&TokenType::Rparen)?;

        let body = Box::new(self.parse_stmt()?);

        Ok(Stmt::For {
            init,
            cond,
            step,
            body,
        })

    }
    
    fn parse_asm_section(&self, asm: &str) -> Result<(String, String), String> {
        let asm = asm.to_string();
        let trimmed = asm.trim_start();

        // default: goes to text section
        if !trimmed.starts_with('$') {
            return Ok((asm, "text".to_string()));
        }

        // remove '$'
        let rest = &trimmed[1..];

        // find first delimiter: space OR newline
        let space_idx = rest.find(' ');
        let newline_idx = rest.find('\n');

        let end_idx = match (space_idx, newline_idx) {
            (Some(s), Some(n)) => std::cmp::min(s, n),
            (Some(s), None) => s,
            (None, Some(n)) => n,
            (None, None) => rest.len(), // only section name present
        };

        let section = rest[..end_idx].to_string();

        // skip delimiter if present
        let remaining = if end_idx < rest.len() {
            rest[end_idx..].trim_start()
        } else {
            ""
        };

        Ok((remaining.to_string(), section))
    }
    fn parse_asm_decl(&self, text: &str) -> Result<AssemblyDecl, String> {
    
        let (raw_code, meta) = match text.find("}$") {
            Some(idx) => {
                let code = text[..idx].to_string();
                let meta = text[idx + 2..].trim().to_string();
                (code, meta)
            }
            None => {
                let (code, section) = self.parse_asm_section(text)?;
                return Ok(AssemblyDecl {
                    code,
                    section,
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                });
            }
        };
    
        let (code, section) = self.parse_asm_section(&raw_code)?;
    
        let inputs = self.parse_asm_inputs(&meta)?;
        let outputs = self.parse_asm_outputs(&meta)?;
    
        Ok(AssemblyDecl {
            code,
            section,
            inputs,
            outputs,
        })
    }
    fn parse_asm_inputs(&self, meta: &str) -> Result<Vec<AsmInput>, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_asm_inputs";
    
        let Some(start_idx) = meta.find("in(") else {
            return Ok(Vec::new());
        };
    
        let rest = &meta[start_idx + 3..];
        let Some(end_idx) = rest.find(')') else {
            return self.error(ERROR_STRING_ROOT, "Unterminated in(...) list");
        };
    
        let inner = rest[..end_idx].trim();
    
        if inner.is_empty() {
            return Ok(Vec::new());
        }
    
        let mut inputs = Vec::new();
    
        for part in inner.split(',') {
            let piece = part.trim();
    
            let Some(arrow_idx) = piece.find("->") else {
                return self.error(ERROR_STRING_ROOT, &format!("Expected '->' in input binding '{}'", piece));
            };
    
            let name = piece[..arrow_idx].trim().to_string();
            let reg = piece[arrow_idx + 2..].trim().to_string();
    
            if name.is_empty() || reg.is_empty() {
                return self.error(ERROR_STRING_ROOT, &format!("Invalid input binding '{}'", piece));
            }
    
            inputs.push(AsmInput { name, reg });
        }
    
        Ok(inputs)
    }
    fn parse_asm_outputs(&self, meta: &str) -> Result<Vec<AsmOutput>, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_asm_outputs";
    
        let Some(start_idx) = meta.find("out(") else {
            return Ok(Vec::new());
        };
    
        let rest = &meta[start_idx + 4..];
        let Some(end_idx) = rest.find(')') else {
            return self.error(ERROR_STRING_ROOT, "Unterminated out(...) list");
        };
    
        let inner = rest[..end_idx].trim();
    
        if inner.is_empty() {
            return Ok(Vec::new());
        }
    
        let mut outputs = Vec::new();
    
        for part in inner.split(',') {
            let piece = part.trim();
    
            let Some(arrow_idx) = piece.find("->") else {
                return self.error(ERROR_STRING_ROOT, &format!(" Expected '->' in output binding '{}'", piece));
            };
    
            let reg = piece[..arrow_idx].trim().to_string();
            let name = piece[arrow_idx + 2..].trim().to_string();
    
            if reg.is_empty() || name.is_empty() {
                return self.error(ERROR_STRING_ROOT, &format!("Invalid output binding '{}'", piece));
            }
    
            outputs.push(AsmOutput { reg, name });
        }
    
        Ok(outputs)
    }
    
    fn parse_asm_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_asm_stmt";
    
        match &self.get().Value {
            TokenValue::String(text) => {
                let decl = self.parse_asm_decl(text)?;
                self.advance();
                Ok(Stmt::Assembly(decl))
            }
            _ => self.error(ERROR_STRING_ROOT, "Assembly token missing string value")
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn is_valid_assignment_target(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(_) => true,
            Expr::Index { .. } => true,
            Expr::Member { .. } => true,
            Expr::Prefix { op: PrefixOp::Deref, .. } => true,
            _ => false,
        }
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
    
        let left = self.parse_logical_or()?;
    
        let op = if self.matches(&TokenType::Assign) {
            Some(AssignOp::Assign)
        }
        else if self.matches(&TokenType::Add_Assign) {
            Some(AssignOp::AddAssign)
        }
        else if self.matches(&TokenType::Sub_Assign) {
            Some(AssignOp::SubAssign)
        }
        else if self.matches(&TokenType::Mul_Assign) {
            Some(AssignOp::MulAssign)
        }
        else if self.matches(&TokenType::Div_Assign) {
            Some(AssignOp::DivAssign)
        }
        else if self.matches(&TokenType::Mod_Assign) {
            Some(AssignOp::ModAssign)
        }
        else if self.matches(&TokenType::And_Assign) {
            Some(AssignOp::AndAssign)
        }
        else if self.matches(&TokenType::Or_Assign) {
            Some(AssignOp::OrAssign)
        }
        else if self.matches(&TokenType::Xor_Assign) {
            Some(AssignOp::XorAssign)
        }
        else if self.matches(&TokenType::Lshift_Assign) {
            Some(AssignOp::LshiftAssign)
        }
        else if self.matches(&TokenType::Rshift_Assign) {
            Some(AssignOp::RshiftAssign)
        }
        else if self.matches(&TokenType::Not_Assign) {
            Some(AssignOp::NotAssign)
        }
        else {
            None
        };
        
        if let Some(op) = op {
            if !self.is_valid_assignment_target(&left) {
                //WARNING: REMOVED
                //return self.error(ERROR_STRING_ROOT, "Invalid assignment target");
            }
    
            let right = self.parse_assignment()?;
    
            Ok(Expr::Assign {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        }
        else {
            Ok(left)
        }
    }
    
    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_xor()?;
    
        while self.matches(&TokenType::Or_Logical) {
            let right = self.parse_logical_xor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalOr,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn parse_logical_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_and()?;
    
        while self.matches(&TokenType::Xor_Logical) {
            let right = self.parse_logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalXor,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_or()?;
    
        while self.matches(&TokenType::And_Logical) {
            let right = self.parse_bitwise_or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalAnd,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    
    fn parse_bitwise_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_xor()?;
    
        while self.matches(&TokenType::Or_Bitwise) {
            let right = self.parse_bitwise_xor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseOr,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn parse_bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_and()?;
    
        while self.matches(&TokenType::Xor_Bitwise) {
            let right = self.parse_bitwise_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseXor,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn parse_bitwise_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;
    
        while self.matches(&TokenType::And_Bitwise) {
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseAnd,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_relational()?;
    
        loop {
            let op = if self.matches(&TokenType::Eq) {
                Some(BinaryOp::Eq)
            }
            else if self.matches(&TokenType::Neq) {
                Some(BinaryOp::Neq)
            }
            else {
                None
            };
    
            if let Some(op) = op {
                let right = self.parse_relational()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }
    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_shift()?;
    
        loop {
            let op = if self.matches(&TokenType::Lt) {
                Some(BinaryOp::Lt)
            }
            else if self.matches(&TokenType::Gt) {
                Some(BinaryOp::Gt)
            }
            else if self.matches(&TokenType::Lte) {
                Some(BinaryOp::Lte)
            }
            else if self.matches(&TokenType::Gte) {
                Some(BinaryOp::Gte)
            }
            else {
                None
            };
    
            if let Some(op) = op {
                let right = self.parse_shift()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }
    
    fn parse_shift(&mut self) -> Result<Expr, String> {

        let mut expr = self.parse_additive()?;
    
        loop {
            let op = if self.matches(&TokenType::Lshift) {
                Some(BinaryOp::Lshift)
            }
            else if self.matches(&TokenType::Rshift) {
                Some(BinaryOp::Rshift)
            }
            else {
                None
            };
    
            if let Some(op) = op {
                let right = self.parse_additive()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }
    
    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_multiplicative()?;
    
        loop {
            let op = if self.matches(&TokenType::Add) {
                Some(BinaryOp::Add)
            }
            else if self.matches(&TokenType::Sub) {
                Some(BinaryOp::Sub)
            }
            else {
                None
            };
    
            if let Some(op) = op {
                let right = self.parse_multiplicative()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }
    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
    
        loop {
            let op = if self.matches(&TokenType::Asterisk) {
                Some(BinaryOp::Mul)
            }
            else if self.matches(&TokenType::Div) {
                Some(BinaryOp::Div)
            }
            else if self.matches(&TokenType::Mod) {
                Some(BinaryOp::Mod)
            }
            else {
                None
            };
    
            if let Some(op) = op {
                let right = self.parse_unary()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.matches(&TokenType::Add) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Plus,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Sub) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Neg,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Not_Logical) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::LogicalNot,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Not_Bitwise) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::BitwiseNot,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Inc) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Inc,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Dec) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Dec,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::And_Bitwise) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Ref,
                expr: Box::new(expr),
            });
        }
    
        if self.matches(&TokenType::Asterisk) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Prefix {
                op: PrefixOp::Deref,
                expr: Box::new(expr),
            });
        }

        if self.is_cast() {
            self.expect(&TokenType::Lparen)?;
            let Type = self.parse_type()?;
            self.expect(&TokenType::Rparen)?;
        
            let expr = self.parse_unary()?;
        
            return Ok(Expr::Cast {
                Type,
                expr: Box::new(expr),
            });
        }

        self.parse_postfix()
    }
    
    fn parse_postfix(&mut self) -> Result<Expr, String> {
    
        let mut expr = self.parse_primary()?;
    
        loop {
            if self.matches(&TokenType::Lparen) {
                let mut args = Vec::new();
    
                if !self.check(&TokenType::Rparen) {
                    loop {
                        args.push(self.parse_expr()?);
    
                        if self.matches(&TokenType::Comma) {
                            continue;
                        }
    
                        break;
                    }
                }
    
                self.expect(&TokenType::Rparen)?;
    
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            }
            else if self.matches(&TokenType::Lbracket) {
                let index = self.parse_expr()?;
                self.expect(&TokenType::Rbracket)?;
    
                expr = Expr::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                };
            }
            else if self.matches(&TokenType::Period) {
                let field = self.consume_identifier()?;
    
                expr = Expr::Member {
                    base: Box::new(expr),
                    field,
                };
            }
            else if self.matches(&TokenType::Inc) {
                expr = Expr::Postfix {
                    op: PostfixOp::Inc,
                    expr: Box::new(expr),
                };
            }
            else if self.matches(&TokenType::Dec) {
                expr = Expr::Postfix {
                    op: PostfixOp::Dec,
                    expr: Box::new(expr),
                };
            }
            else {
                break;
            }
        }
    
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expr, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_primary";
    
        match self.get().Type {
            TokenType::True_Keyword => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            TokenType::False_Keyword => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            TokenType::Int_Literal => {
                match &self.get().Value {
                    TokenValue::Int(v) => {
                        let out = *v;
                        self.advance();
                        Ok(Expr::IntLiteral(out))
                    }
                    _ => self.error(ERROR_STRING_ROOT, "Int literal missing int value")
                }
            }
            TokenType::Float_Literal => {
                match &self.get().Value {
                    TokenValue::Float(v) => {
                        let out = *v;
                        self.advance();
                        Ok(Expr::FloatLiteral(out))
                    }
                    _ => self.error(ERROR_STRING_ROOT, "Float literal missing float value")
                }
            }
            TokenType::Char_Literal => {
                match &self.get().Value {
                    TokenValue::Char(v) => {
                        let out = *v;
                        self.advance();
                        Ok(Expr::CharLiteral(out))
                    }
                    _ => self.error(ERROR_STRING_ROOT, "Char literal missing char value")
                }
            }
            TokenType::String_Literal => {
                match &self.get().Value {
                    TokenValue::String(text) => {
                        let out = text.clone();
                        self.advance();
                        Ok(Expr::StringLiteral(out))
                    }
                    _ => self.error(ERROR_STRING_ROOT, "String literal missing string value")
                }
            }
            TokenType::Identifier => {
                match &self.get().Value {
                    TokenValue::String(name) => {
                        let out = name.clone();
                        self.advance();
                        Ok(Expr::Identifier(out))
                    }
                    _ => self.error(ERROR_STRING_ROOT, "Identifier token missing string value")
                }
            }
            TokenType::Lparen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenType::Rparen)?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => self.error(ERROR_STRING_ROOT, "Expected expression")
        }
    }

    fn parse_array_suffixes(&mut self, base: TypeName) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_array_suffixes";
    
        let mut dims: Vec<usize> = Vec::new();
    
        while self.matches(&TokenType::Lbracket) {
            let size_token = self.expect(&TokenType::Int_Literal)?;
    
            let size = match size_token.Value {
                TokenValue::Int(v) => {
                    if v <= 0 {
                        return self.error(
                            ERROR_STRING_ROOT,
                            "Array size must be greater than zero",
                        );
                    }
    
                    v as usize
                }
    
                _ => {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Expected integer literal as array size",
                    );
                }
            };
    
            self.expect(&TokenType::Rbracket)?;
    
            dims.push(size);
        }
    
        let mut ty = base;
    
        for dim in dims.into_iter().rev() {
            ty = TypeName::Array(Box::new(ty), dim);
        }
    
        Ok(ty)
    }

    fn error<T>(&self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \nCurrent token:\n\tType {:?}\n\tPosition {}:{}", self.get().Type, self.get().Span.row, self.get().Span.col))
    }

}
