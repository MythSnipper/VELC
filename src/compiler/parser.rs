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
    Assembly(String),
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

    Assembly(String),
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
            Err(format!("{ERROR_STRING_ROOT}:Expected {:?} token", expected))
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
            _ => return Err(format!("{ERROR_STRING_ROOT}:Expected type keyword")),
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
            return Err(format!("{ERROR_STRING_ROOT}:Current token is not an identifier"));
        }
    
        match &self.get().Value {
            TokenValue::String(s) => {
                let out = s.clone();
                self.advance();
                Ok(out)
            }
            _ => Err(format!("{ERROR_STRING_ROOT}:Identifier token missing string value")),
        }
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_program";
        let mut program = Program::new();
        while !self.is_at_end() {
            let top_level = self.parse_top_level()?;
            program.items.push(top_level);
        }
        Ok(program)
    }
    fn parse_top_level(&mut self) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_top_level";
        //if top level is asm
        if self.check(&TokenType::Assembly) {
            return self.parse_top_level_assembly();
        }
        //parse two outcomes of top level
        // either
        // [typename] [identifier](   -> Function
        // or
        // [typename] [identifier] =  -> GlobalVar
        if self.is_type_keyword() {
            let Type = self.parse_type()?;
            let id = self.consume_identifier()?;

            if self.check(&TokenType::Lparen) {
                return self.parse_function(Type, id);
            }
            else {
                return self.parse_global_var_decl(Type, id);
            }
        }
        Err(format!("{ERROR_STRING_ROOT}:Expected type keyword"))
    }
    fn parse_top_level_assembly(&mut self) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_top_level_assembly";
        match &self.get().Value {
            TokenValue::String(text) => {
                let out = text.clone();
                self.advance();
                Ok(TopLevel::Assembly(out))
            }
            _ => Err(format!("{ERROR_STRING_ROOT}:Assembly token missing string value"))
        }
    }
    fn parse_function(&mut self, t: TypeName, id: String) -> Result<TopLevel, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_function";
        
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
            Some(self.parse_expr()?)
        }
        else {
            None
        };
    
        if !self.matches(&TokenType::Semicolon) {
            Err(format!("{ERROR_STRING_ROOT}:Expected semicolon"))
        }
        else {
            Ok(TopLevel::GlobalVar(VarDecl {
                Type: t,
                name: id,
                init: init,
            }))
        }
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_block";
        
        let mut stmts: Vec<Stmt> = Vec::new();

        self.expect(&TokenType::Lbrace)?;

        while !self.check(&TokenType::Rbrace) {
            if self.is_at_end() {
                return Err(format!("{ERROR_STRING_ROOT}:expected }}"));
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenType::Rbrace)?;

        Ok(Stmt::Block(stmts))
    }
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_stmt";

        if self.is_at_end() {
            return Err(format!("{ERROR_STRING_ROOT}:Unexpected EOF"));
        }

        let stmt = if let Ok(typename) = self.parse_type() {
            //VarDecl
            return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
        }
        else if self.check(&TokenType::Lbrace) {
            //Block
            self.parse_block()?
        }
        else {
            match self.get().Type {
                TokenType::If_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::Else_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::While_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::For_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::Return_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::Break_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                TokenType::Continue_Keyword => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
                _ => {
                    return Err(format!("{ERROR_STRING_ROOT}:UNIMPEMENTED"));
                }
            }
        };

        Ok(stmt)
    }
    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_return_stmt";

        self.expect(&TokenType::Return_Keyword)?;

        let expr = self.parse_expr()?;

        self.expect(&TokenType::Semicolon)?;

        Ok(expr)
    }
    fn parse_var_decl_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_var_decl_stmt";
        
        let t = self.parse_type()?;
        let id = self.consume_identifier()?;

        let init = if self.matches(&TokenType::Assign) {
            Some(self.parse_expr()?)
        }
        else {
            None
        };
    
        if !self.matches(&TokenType::Semicolon) {
            Err(format!("{ERROR_STRING_ROOT}:Expected semicolon"))
        }
        else {
            Ok(Stmt::VarDecl(VarDecl {
                Type: t,
                name: id,
                init: init,
            }))
        }

    }
    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let ERROR_STRING_ROOT = "velc:Parser:parse_var_decl_stmt";
        let expr = self.parse_expr()?;

        self.expect(&TokenType::Semicolon)?;

        Ok(expr)
    }



}

