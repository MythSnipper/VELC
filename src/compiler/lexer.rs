


#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {    

    //keywords
    Bool_Keyword,
    Int8_Keyword,
    Int16_Keyword,
    Int32_Keyword,
    Int64_Keyword,
    Uint8_Keyword,
    Uint16_Keyword,
    Uint32_Keyword,
    Uint64_Keyword,
    Char_Keyword,
    Float32_Keyword,
    Float64_Keyword,
    String_Keyword,
    Void_Keyword,
    
    If_Keyword,
    Else_Keyword,
    While_Keyword,
    For_Keyword,
    Return_Keyword,
    Break_Keyword,
    Continue_Keyword,

    True_Keyword,
    False_Keyword,

    //identifiers and literals
    Identifier,
    Int_Literal,
    Float_Literal,
    Char_Literal,
    String_Literal,
    Assembly,

    //punctuation
    Semicolon,  // ;
    Comma,      // ,
    Lparen,     // (
    Rparen,     // )
    Lbrace,     // {
    Rbrace,     // }
    Lbracket,   // [
    Rbracket,   // ]
    Period,     // .

    //operators
    Assign,               // =
    Add_Assign,           // +=
    Sub_Assign,           // -=
    Mul_Assign,           // *=
    Div_Assign,           // /=
    Mod_Assign,           // %=
    And_Assign,           // &=
    Or_Assign,            // |=
    Xor_Assign,           // ^=
    Lshift_Assign,        // <<=
    Rshift_Assign,        // >>=
    Not_Assign,           // ~=

    Add,                  // +
    Sub,                  // -
    Asterisk,             // *
    Div,                  // /
    Mod,                  // %

    Eq,                   // ==
    Neq,                  // !=
    Lt,                   // <
    Gt,                   // >
    Lte,                  // <=
    Gte,                  // >=

    And_Bitwise,          // &
    And_Logical,          // &&
    Or_Bitwise,           // |
    Or_Logical,           // ||
    Xor_Bitwise,          // ^
    Xor_Logical,          // ^^
    Lshift,               // <<
    Rshift,               // >>

    //prefix
    Not_Logical,          // !
    Not_Bitwise,          // ~
    Inc,                  // ++
    Dec,                  // --

    //postfix
    //Inc
    //Dec

    //special
    None,
    EOF
}
impl Default for TokenType {
    fn default() -> Self {
        Self::EOF
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    None,
    String(String),
    Int(i64),
    Float(f64),
    Char(char),
}
impl Default for TokenValue {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TokenSpan {
    pub row: usize,
    pub col: usize
}

#[derive(Default, Debug, Clone)]
pub struct Token {
    pub Type: TokenType,
    pub Value: TokenValue,
    pub Span: TokenSpan
}

#[derive(Default, Debug, Clone)]
pub struct Lexer {
    source: Vec<char>,
    index: usize,
    row: usize,
    col: usize,
    tokens: Vec<Token>,
}
impl Lexer {
    pub fn new() -> Self {
        Self {
            source: Vec::new(),
            tokens: Vec::new(),
            index: 0,
            row: 1,
            col: 1
        }
    }
    pub fn from_source(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            index: 0,
            row: 1,
            col: 1
        }
    }
    pub fn run(&mut self) -> Result<Vec<Token>, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:run";
        
        while !self.is_at_end() {
            let ch = self.get();
            let next_ch = self.next();
            let next_next_ch = self.next_next();

            let new_token = match ch {
                //skip whitespace
                ' ' | '\n' | '\t' => {
                    self.whitespace()?
                }
                //comments
                '/' if next_ch == '/' => { //single line comment //
                    self.scan_comment_single()?
                }
                '/' if next_ch == '{' => { //multi line comment /{  }/
                    self.scan_comment_multi()?
                }

                //assembly
                '%' if next_ch == '%' => { //single line assembly %%
                    self.scan_asm_single()?
                }
                '%' if next_ch == '{' => { //multi line assembly %{ }%
                    self.scan_asm_multi()?
                }

                //keyword or identifier
                'a'..='z' | '_' => { //all keywords and identifiers
                    self.scan_keyword_or_identifier()?
                }

                //literals
                '0'..='9' => {
                    self.scan_num_literal()?
                }
                '\'' => {
                    self.scan_char_literal()?
                }
                '\"' => {
                    self.scan_string_literal()?
                }

                //punctuation
                ';' | ',' | '(' | ')' | '{' | '}' | '[' | ']' | '.'=> {
                    self.scan_punctuation()?
                }
                
                //operators
                '=' if next_ch == '=' => { //Eq
                    self.scan_operator(TokenType::Eq, 2)?
                }
                '=' => { //Assign
                    self.scan_operator(TokenType::Assign, 1)?
                }
                '+' if next_ch == '=' => { //Add_Assign
                    self.scan_operator(TokenType::Add_Assign, 2)?
                }
                '+' if next_ch == '+' => { //Inc
                    self.scan_operator(TokenType::Inc, 2)?
                }
                '+' => { //Add
                    self.scan_operator(TokenType::Add, 1)?
                }
                '-' if next_ch == '=' => { //Sub_Assign
                    self.scan_operator(TokenType::Sub_Assign, 2)?
                }
                '-' if next_ch == '-' => { //Dec
                    self.scan_operator(TokenType::Dec, 2)?
                }
                '-' => { //Sub
                    self.scan_operator(TokenType::Sub, 1)?
                }
                '*' if next_ch == '=' => { //Mul_Assign
                    self.scan_operator(TokenType::Mul_Assign, 2)?
                }
                '*' => { //Asterisk
                    self.scan_operator(TokenType::Asterisk, 1)?
                }
                '/' if next_ch == '=' => { //Div_Assign
                    self.scan_operator(TokenType::Div_Assign, 2)?
                }
                '/' => { //Div
                    self.scan_operator(TokenType::Div, 1)?
                }
                '%' if next_ch == '=' => { //Mod_Assign
                    self.scan_operator(TokenType::Mod_Assign, 2)?
                }
                '%' => { //Mod
                    self.scan_operator(TokenType::Mod, 1)?
                }
                '&' if next_ch == '=' => { //And_Assign
                    self.scan_operator(TokenType::And_Assign, 2)?
                }
                '&' if next_ch == '&' => { //And_Logical
                    self.scan_operator(TokenType::And_Logical, 2)?
                }
                '&' => { //And_Bitwise
                    self.scan_operator(TokenType::And_Bitwise, 1)?
                }
                '|' if next_ch == '=' => { //Or_Assign
                    self.scan_operator(TokenType::Or_Assign, 2)?
                }
                '|' if next_ch == '|' => { //Or_Logical
                    self.scan_operator(TokenType::Or_Logical, 2)?
                }
                '|' => { //Or_Bitwise
                    self.scan_operator(TokenType::Or_Bitwise, 1)?
                }
                '^' if next_ch == '=' => { //Xor_Assign
                    self.scan_operator(TokenType::Xor_Assign, 2)?
                }
                '^' if next_ch == '^' => { //Xor_Logical
                    self.scan_operator(TokenType::Xor_Logical, 2)?
                }
                '^' => { //Xor_Bitwise
                    self.scan_operator(TokenType::Xor_Bitwise, 1)?
                }
                '<' if next_ch == '<' && next_next_ch == '=' => { //Lshift_Assign
                    self.scan_operator(TokenType::Lshift_Assign, 3)?
                }
                '<' if next_ch == '<' => { //Lshift
                    self.scan_operator(TokenType::Lshift, 2)?
                }
                '<' if next_ch == '=' => { //Lte
                    self.scan_operator(TokenType::Lte, 2)?
                }
                '<' => { //Lt
                    self.scan_operator(TokenType::Lt, 1)?
                }
                '>' if next_ch == '>' && next_next_ch == '=' => { //Rshift_Assign
                    self.scan_operator(TokenType::Rshift_Assign, 3)?
                }
                '>' if next_ch == '>' => { //Rshift
                    self.scan_operator(TokenType::Rshift, 2)?
                }
                '>' if next_ch == '=' => { //Gte
                    self.scan_operator(TokenType::Gte, 2)?
                }
                '>' => { //Gt
                    self.scan_operator(TokenType::Gt, 2)?
                }
                '~' if next_ch == '=' => { //Not_Assign
                    self.scan_operator(TokenType::Not_Assign, 2)?
                }
                '~' => { //Not_Bitwise
                    self.scan_operator(TokenType::Not_Bitwise, 1)?
                }
                '!' if next_ch == '=' => { //Neq
                    self.scan_operator(TokenType::Neq, 2)?
                }
                '!' => { //Not_Logical
                    self.scan_operator(TokenType::Not_Logical, 1)?
                }
                _ => {
                    return self.error(ERROR_STRING_ROOT, "Invalid character");
                }
            };
            if new_token.Type != TokenType::None {
                println!("\tnew token: {:?}:{:?}:{}:{}", new_token.Type, new_token.Value, new_token.Span.row, new_token.Span.col);
                self.tokens.push(new_token);
            } 
        }
        println!("EOF!");
        self.tokens.push(Token{Type: TokenType::EOF, Value: TokenValue::None, Span: TokenSpan{row: self.row, col: self.col}});
        
        
        Ok(self.tokens.clone())
    }
    
    fn advance(&mut self) {
        if self.is_at_end() {
            return;
        }
        if self.source[self.index] == '\n' {
            self.row += 1;
            self.col = 1;
        }
        else{
            self.col += 1;
        }
        self.index += 1;
    }
    fn next(&self) -> char {
        if let Some(c) = self.source.get(self.index+1) {
            c.to_owned()
        }
        else {
            '\0'
        }
    }
    fn next_next(&self) -> char {
        if let Some(c) = self.source.get(self.index+2) {
            c.to_owned()
        }
        else{
            '\0'
        }
    }
    fn is_at_end(&self) -> bool {
        self.index >= self.source.len()
    }
    fn get(&self) -> char {
        self.source.get(self.index).unwrap().to_owned()
    }

    fn scan_keyword_or_identifier(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_keyword_or_identifier";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;
        
        let mut text: String = String::new();

        while !self.is_at_end() {
            let c = self.get();
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
            }
            else {
                new_token.Type = match text.as_str() {
                    "bool" => TokenType::Bool_Keyword,
                    "int8" => TokenType::Int8_Keyword,
                    "int16" => TokenType::Int16_Keyword,
                    "int32" => TokenType::Int32_Keyword,
                    "int64" => TokenType::Int64_Keyword,
                    "uint8" => TokenType::Uint8_Keyword,
                    "uint16" => TokenType::Uint16_Keyword,
                    "uint32" => TokenType::Uint32_Keyword,
                    "uint64" => TokenType::Uint64_Keyword,
                    "char" => TokenType::Char_Keyword,
                    "float32" => TokenType::Float32_Keyword,
                    "float64" => TokenType::Float64_Keyword,
                    "string" => TokenType::String_Keyword,
                    "void" => TokenType::Void_Keyword,
                    
                    "if" => TokenType::If_Keyword,
                    "else" => TokenType::Else_Keyword,
                    "while" => TokenType::While_Keyword,
                    "for" => TokenType::For_Keyword,
                    "return" => TokenType::Return_Keyword,
                    "break" => TokenType::Break_Keyword,
                    "continue" => TokenType::Continue_Keyword,
                    "true" => TokenType::True_Keyword,
                    "false" => TokenType::False_Keyword,
                    
                    _ => TokenType::Identifier
                };
                break;
            }
            self.advance();
        }

        if text.is_empty() {
            //nothing scanned, error
            self.error(ERROR_STRING_ROOT, "Nothing scanned")
        }
        else{
            if new_token.Type == TokenType::Identifier {
                //add identifier name to value
                new_token.Value = TokenValue::String(text.clone());
            }
            Ok(new_token)
        }
    }
    fn scan_num_literal(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_num_literal";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        let mut text = String::new();
        let mut is_float = false;

        while let Some(c) = self.source.get(self.index) {
            let ch = c.to_owned();
            match c {
                '0'..='9' => {
                    text.push(ch);
                    self.advance();
                }
                '_' => {
                    self.advance();
                }
                '.' if !is_float => {
                    text.push(ch);
                    is_float = true;
                    self.advance();
                }
                _ => {
                    break;
                }
            }
        }

        if text.is_empty() {
            self.error(ERROR_STRING_ROOT, "Nothing scanned")
        }
        else{
            new_token.Type = if is_float {
                TokenType::Float_Literal
            }
            else {
                TokenType::Int_Literal
            };

            new_token.Value = if is_float {
                let value = text.parse::<f64>();
                if let Err(e) = value {
                    return self.error(ERROR_STRING_ROOT, &format!("{:?}", e));
                }
                TokenValue::Float(value.unwrap())
            } else {
                let value = text.parse::<i64>();
                if let Err(e) = value {
                    return self.error(ERROR_STRING_ROOT, &format!("{:?}", e));
                }
                TokenValue::Int(value.unwrap())
            };
            Ok(new_token)
        }
    }
    fn scan_string_literal(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_string_literal";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        let mut text = String::new();

        if self.get() != '"' {
            return self.error(ERROR_STRING_ROOT, "Expected opening \" for string literal");
        }
        self.advance();

        while !self.is_at_end() {
            let ch = self.get();
            match ch {
                '"' => {
                    self.advance();
                    new_token.Type = TokenType::String_Literal;
                    new_token.Value = TokenValue::String(text);
                    return Ok(new_token);
                }
                '\\' => {
                    self.advance();

                    if let Some(nextchar) = self.source.get(self.index) {

                        let escaped = match nextchar {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            'f' => '\x0C',
                            'b' => '\x08',
                            '0' => '\0',
                            '\\' => '\\',
                            '"' => '"',
                            '\'' => '\'',
                            other => other.to_owned(),
                        };
                        text.push(escaped);
                    }
                    else {
                        return self.error(ERROR_STRING_ROOT, "Unterminated escape sequence");
                        
                    }
                    self.advance();
                }
                '\n' => {
                    return self.error(ERROR_STRING_ROOT, "Unterminated string literal");
                }
                _ => {
                    text.push(ch);
                    self.advance();
                }
            }
        }
        return self.error(ERROR_STRING_ROOT, "Expected closing \" for string literal");
    }
    fn scan_char_literal(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_char_literal";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        let text: char;

        if self.get() != '\'' {
            return self.error(ERROR_STRING_ROOT, "Expected opening \' for char literal");
        }
        self.advance();
        if self.is_at_end() {
            return self.error(ERROR_STRING_ROOT, "Expected closing \' for char literal");
        }

        let ch = self.get();
        match ch {
            '\'' => {
                return self.error(ERROR_STRING_ROOT, "No char scanned");
            }
            '\\' => {
                self.advance();

                if let Some(nextchar) = self.source.get(self.index) {
                    let escaped = match nextchar {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        'f' => '\x0C',
                        'b' => '\x08',
                        '0' => '\0',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        other => other.to_owned(),
                    };
                    text = escaped;
                }
                else {
                    return self.error(ERROR_STRING_ROOT, "Unterminated escape sequence");
                }
                self.advance();
            }
            '\n' | '\t' | '\r' => {
                return self.error(ERROR_STRING_ROOT, "Unterminated char literal");
            }
            _ => {
                text = ch;
                self.advance();
            }
        }

        if self.is_at_end() {
            return self.error(ERROR_STRING_ROOT, "Expected closing \' for char literal");
        }

        let ch = self.get();
        match ch {
            '\'' => {
                self.advance();
                new_token.Type = TokenType::Char_Literal;
                new_token.Value = TokenValue::Char(text);
                return Ok(new_token);
            }
            '\n' => {
                return self.error(ERROR_STRING_ROOT, "Unterminated char literal");
            }
            _ => {
                return self.error(ERROR_STRING_ROOT, "Character literal cannot have multiple characters");
            }
        }
    }

    fn scan_asm_single(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_asm_single";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        let mut text: String = String::new();

        self.advance();
        self.advance();
        while !self.is_at_end() {
            let ch = self.get();
            match ch {
                '\n' => {
                    text.push(ch);
                    self.advance();
                    new_token.Type = TokenType::Assembly;
                    new_token.Value = TokenValue::String(text);
                    return Ok(new_token);
                }
                _ => {
                    text.push(ch);
                }
            }
            self.advance();
        }
        new_token.Type = TokenType::Assembly;
        new_token.Value = TokenValue::String(text);
        Ok(new_token)
    }
    fn scan_asm_multi(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_asm_multi";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        let mut text: String = String::new();

        self.advance();
        self.advance();
        while !self.is_at_end() {
            let ch = self.get();
            let next_ch = self.next();
            if ch == '}' && next_ch == '%' {
                self.advance();
                self.advance();
                new_token.Type = TokenType::Assembly;
                new_token.Value = TokenValue::String(text);
                return Ok(new_token);
            }
            else{
                text.push(ch)
            }
            self.advance();
        }
        self.error(ERROR_STRING_ROOT, "Unterminated multiline assembly")
    }

    fn scan_comment_single(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_comment_single";

        self.advance();
        self.advance();
        while !self.is_at_end() {
            let ch = self.source.get(self.index).unwrap();
            match ch {
                '\n' => {
                    self.advance();
                    return Ok(Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan::default()});
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan::default()})
    }
    fn scan_comment_multi(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_comment_multi";

        self.advance();
        self.advance();
        while !self.is_at_end() {
            let ch = self.get();
            let next_ch = self.next();
            if ch == '}' && next_ch == '/' {
                self.advance();
                self.advance();
                return Ok(Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan::default()});
            }
            else{
                self.advance();
            }
        }
        self.error(ERROR_STRING_ROOT, "Unterminated multiline comment")
    }

    fn scan_punctuation(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_punctuation";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;

        new_token.Type = match self.get() {
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '(' => TokenType::Lparen,
            ')' => TokenType::Rparen,
            '{' => TokenType::Lbrace,
            '}' => TokenType::Rbrace,
            '[' => TokenType::Lbracket,
            ']' => TokenType::Rbracket,
            '.' => TokenType::Period,
            _ => {
                return self.error(ERROR_STRING_ROOT, "Invalid punctuation scanned");
            }
        };

        new_token.Value = TokenValue::None;

        self.advance();

        Ok(new_token)
    }
    fn scan_operator(&mut self, t: TokenType, skips: usize) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:scan_operator";

        let mut new_token: Token = Token::default();
        new_token.Span.row = self.row;
        new_token.Span.col = self.col;
        
        new_token.Type = t;

        new_token.Value = TokenValue::None;

        for _ in 0..skips {
            self.advance();
        }

        Ok(new_token)
    }

    fn whitespace(&mut self) -> Result<Token, String> {
        let ERROR_STRING_ROOT = "velc:Lexer:whitespace";
        self.advance();
        Ok(Token{Type: TokenType::None, Value: TokenValue::None, Span: TokenSpan::default()})
    }

    fn error<T>(&mut self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err}\nCurrent character:\n\tPosition {}:{}", self.row, self.col))
    }
}
