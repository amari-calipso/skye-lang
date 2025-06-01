use std::collections::HashMap;
use std::rc::Rc;

use crate::tokens::{Token, TokenType};
use crate::utils::{error, is_alpha, is_alphanumeric, is_beginning_digit, is_bin_digit, is_digit, is_hex_digit, is_oct_digit, substring};

pub struct Scanner<'a> {
    source: &'a String,
    filename: Rc<str>,
    pub tokens: Vec<Token>,
    start_positions: Vec<usize>,

    keywords: HashMap<&'static str, TokenType>,

    start: usize,
    curr:  usize,
    line:  usize,

    pub had_error: bool
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String, filename: Rc<str>) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert(       "as", TokenType::As);
        keywords.insert(       "do", TokenType::Do);
        keywords.insert(       "fn", TokenType::Fn);
        keywords.insert(       "if", TokenType::If);
        keywords.insert(      "for", TokenType::For);
        keywords.insert(      "let", TokenType::Let);
        keywords.insert(      "try", TokenType::Try);
        keywords.insert(      "use", TokenType::Use);
        keywords.insert(     "else", TokenType::Else);
        keywords.insert(     "enum", TokenType::Enum);
        keywords.insert(     "impl", TokenType::Impl);
        keywords.insert(     "void", TokenType::Void);
        keywords.insert(    "break", TokenType::Break);
        keywords.insert(    "const", TokenType::Const);
        keywords.insert(    "defer", TokenType::Defer);
        keywords.insert(    "macro", TokenType::Macro);
        keywords.insert(    "union", TokenType::Union);
        keywords.insert(    "while", TokenType::While);
        keywords.insert(   "import", TokenType::Import);
        keywords.insert(   "return", TokenType::Return);
        keywords.insert(   "struct", TokenType::Struct);
        keywords.insert(   "switch", TokenType::Switch);
        keywords.insert(  "default", TokenType::Default);
        keywords.insert( "bitfield", TokenType::Bitfield);
        keywords.insert( "continue", TokenType::Continue);
        keywords.insert("interface", TokenType::Interface);
        keywords.insert("namespace", TokenType::Namespace);

        // reserved for internal usage
        keywords.insert(          "_DOT_", TokenType::Reserved);
        keywords.insert(        "_FNPTR_", TokenType::Reserved);
        keywords.insert(        "_GENOF_", TokenType::Reserved);
        keywords.insert(        "_PTROF_", TokenType::Reserved);
        keywords.insert(       "_GENAND_", TokenType::Reserved);
        keywords.insert(       "_GENEND_", TokenType::Reserved);
        keywords.insert(       "_PTREND_", TokenType::Reserved);
        keywords.insert(      "_UNKNOWN_", TokenType::Reserved);
        keywords.insert(     "SKYE_ENUM_", TokenType::Reserved);
        keywords.insert(    "_PARAM_AND_", TokenType::Reserved);
        keywords.insert(    "_PARAM_END_", TokenType::Reserved);
        keywords.insert(    "_FNPTR_END_", TokenType::Reserved);
        keywords.insert(    "SKYE_UNION_", TokenType::Reserved);
        keywords.insert(   "SKYE_STRING_", TokenType::Reserved);
        keywords.insert(   "SKYE_STRUCT_", TokenType::Reserved);
        keywords.insert("SKYE_ENUM_INIT_", TokenType::Reserved);

        Scanner {
            source, filename, tokens: Vec::new(), keywords, start_positions: Vec::new(),
            start: 0, curr: 0, line: 0, had_error: false
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.curr).unwrap();
        self.curr += 1;
        c
    }

    fn add_token(&mut self, type_: TokenType) {
        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()), 
            Rc::clone(&self.filename), type_, 
            substring(&self.source, self.start, self.curr).into(), 
            self.start - self.start_positions[self.line], 
            self.curr - self.start_positions[self.line], self.line
        ));
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.curr).unwrap() != expected {
            return false;
        }

        self.curr += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.curr).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.curr + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.curr + 1).unwrap()
        }
    }

    fn error(&mut self, msg: &str) {
        error(
            &Rc::from(self.source.as_ref()), msg, 
            &self.filename,
            self.start - self.start_positions[self.line], 
            self.curr - self.start, self.line
        );
        
        self.had_error = true;
    }

    fn string_char_literal(&mut self, ch: char, type_: TokenType) {
        let mut back_slash = false;
        loop {
            let c = self.peek();
            if self.is_at_end() || (c == ch && !back_slash) {
                break;
            }

            let old_backslash = back_slash;
            back_slash = false;

            match c {
                '\n' => self.line += 1,
                '\\' => {
                    if !old_backslash {
                        back_slash = true;
                    }
                }
                _ => (),
            }

            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string");
            return;
        }

        self.advance();

        self.start += 1;
        self.curr -= 1;

        self.add_token(type_);

        self.start -= 1;
        self.curr += 1;
    }
    
    fn float_type(&mut self) {
        let c0 = self.advance();
        let c1 = self.advance();

        if c0 == '3' && c1 == '2' {
            self.curr -= 3;
            self.add_token(TokenType::F32);
            self.curr += 3;
        } else if c0 == '6' && c1 == '4' {
            self.curr -= 3;
            self.add_token(TokenType::F64);
            self.curr += 3;
        } else {
            self.error("Unrecognized type notation");
        }
    }

    fn unsigned_int_type(&mut self) {
        self.advance();

        let c0 = self.advance();
        if c0 == '8' {
            self.curr -= 2;
            self.add_token(TokenType::U8);
            self.curr += 2;
            return;
        }

        let c1 = self.advance();
        if c0 == '1' && c1 == '6' {
            self.curr -= 3;
            self.add_token(TokenType::U16);
            self.curr += 3;
        } else if c0 == '3' && c1 == '2' {
            self.curr -= 3;
            self.add_token(TokenType::U32);
            self.curr += 3;
        } else if c0 == '6' && c1 == '4' {
            self.curr -= 3;
            self.add_token(TokenType::U64);
            self.curr += 3;
        } else if c0 == 's' && c1 == 'z' {
            self.curr -= 3;
            self.add_token(TokenType::Usz);
            self.curr += 3;
        } else {
            self.error("Unrecognized type notation");
        }
    }

    fn signed_int_type(&mut self) {
        self.advance();

        let c0 = self.advance();
        if c0 == '8' {
            self.curr -= 2;
            self.add_token(TokenType::I8);
            self.curr += 2;
            return;
        }

        let c1 = self.advance();
        if c0 == '1' && c1 == '6' {
            self.curr -= 3;
            self.add_token(TokenType::I16);
            self.curr += 3;
        } else if c0 == '3' && c1 == '2' {
            self.curr -= 3;
            self.add_token(TokenType::I32);
            self.curr += 3;
        } else if c0 == '6' && c1 == '4' {
            self.curr -= 3;
            self.add_token(TokenType::I64);
            self.curr += 3;
        } else {
            self.error("Unrecognized type notation");
        }
    }

    fn number(&mut self, scan_start: bool) {
        if scan_start {
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let c = self.peek();
        match c {
            '.' => {
                self.advance();

                if is_digit(self.peek()) {
                    loop {
                        self.advance();

                        if !is_digit(self.peek()) {
                            break;
                        }
                    }
                } else {
                    self.error("Expecting digits after decimal point");
                }
                
                if self.match_('f') {
                    self.float_type();
                } else {
                    self.add_token(TokenType::AnyFloat);
                }
            }
            'u' => self.unsigned_int_type(),
            'i' => self.signed_int_type(),
            'f' => {
                self.advance();
                self.float_type();
            }
            _ => self.add_token(TokenType::AnyInt)
        }
    }

    fn alt_base_number(&mut self) {
        let c = self.peek();
        match c {
            'b' => {
                self.advance();

                while is_bin_digit(self.peek()) {
                    self.advance();
                }

                if self.match_('u') {
                    self.unsigned_int_type();
                } else if self.match_('i') {
                    self.signed_int_type();
                } else {
                    self.add_token(TokenType::AnyInt);
                }
            }
            'o' => {
                self.advance();

                while is_oct_digit(self.peek()) {
                    self.advance();
                }

                if self.match_('u') {
                    self.unsigned_int_type();
                } else if self.match_('i') {
                    self.signed_int_type();
                } else {
                    self.add_token(TokenType::AnyInt);
                }
            }
            'x' => {
                self.advance();

                while is_hex_digit(self.peek()) {
                    self.advance();
                }

                if self.match_('u') {
                    self.unsigned_int_type();
                } else if self.match_('i') {
                    self.signed_int_type();
                } else {
                    self.add_token(TokenType::AnyInt);
                }
            }
            _ => {
                if is_digit(c) {
                    loop {
                        self.advance();

                        if !is_digit(self.peek()) {
                            break;
                        }
                    }

                    self.number(false);
                    self.error("Base 10 numbers cannot be zero-padded");
                } else {
                    self.number(false);
                }
            }
        }
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let kind = self.keywords.get(
            substring(&self.source, self.start, self.curr).as_str()
        );

        if let Some(type_) = kind {
            if let TokenType::Reserved = type_ {
                self.error("This keyword is reserved for internal use. Please use a different name");
            } else {
                self.add_token(type_.clone())
            }
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftSquare),
            ']' => self.add_token(TokenType::RightSquare),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            ';' => self.add_token(TokenType::Semicolon),
            '?' => self.add_token(TokenType::Question),
            '@' => self.add_token(TokenType::At),
            '~' => self.add_token(TokenType::Tilde),
            '#' => self.add_token(TokenType::Hash),

            '!' => {
                if self.match_('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '.' => {
                if self.match_('.') {
                    self.add_token(TokenType::DotDot);
                } else {
                    self.add_token(TokenType::Dot);
                }
            }
            '=' => {
                if self.match_('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '*' => {
                if self.match_('=') {
                    self.add_token(TokenType::StarEquals);
                } else {
                    self.add_token(TokenType::Star);
                }
            }
            '%' => {
                if self.match_('=') {
                    self.add_token(TokenType::ModEquals);
                } else {
                    self.add_token(TokenType::Mod);
                }
            }
            '^' => {
                if self.match_('=') {
                    self.add_token(TokenType::XorEquals);
                } else {
                    self.add_token(TokenType::BitwiseXor);
                }
            }
            ':' => {
                if self.match_(':') {
                    self.add_token(TokenType::ColonColon);
                } else {
                    self.add_token(TokenType::Colon);
                }
            }

            '+' => {
                if self.match_('+') {
                    self.add_token(TokenType::PlusPlus);
                } else if self.match_('=') {
                    self.add_token(TokenType::PlusEquals);
                } else {
                    self.add_token(TokenType::Plus);
                }
            }
            '|' => {
                if self.match_('|') {
                    self.add_token(TokenType::LogicOr);
                } else if self.match_('=') {
                    self.add_token(TokenType::OrEquals);
                } else {
                    self.add_token(TokenType::BitwiseOr);
                }
            }
            '&' => {
                if self.match_('&') {
                    self.add_token(TokenType::LogicAnd);
                } else if self.match_('=') {
                    self.add_token(TokenType::AndEquals);
                } else {
                    self.add_token(TokenType::BitwiseAnd);
                }
            }

            '-' => {
                if self.match_('-') {
                    self.add_token(TokenType::MinusMinus);
                } else if self.match_('=') {
                    self.add_token(TokenType::MinusEquals);
                } else if self.match_('>') {
                    self.add_token(TokenType::Arrow);
                } else {
                    self.add_token(TokenType::Minus);
                }
            }
            
            '<' => {
                if self.match_('<') {
                    if self.match_('=') {
                        self.add_token(TokenType::ShiftLeftEquals);
                    } else {
                        self.add_token(TokenType::ShiftLeft);
                    }
                } else if self.match_('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.match_('>') {
                    if self.match_('=') {
                        self.add_token(TokenType::ShiftRightEquals);
                    } else {
                        self.add_token(TokenType::ShiftRight);
                    }
                } else if self.match_('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }

            '/' => {
                if self.match_('=') {
                    self.add_token(TokenType::SlashEquals);
                } else if self.match_('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_('*') {
                    while !(
                        (self.peek() == '*' && self.peek_next() == '/') ||
                        self.is_at_end()
                    ) {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            '"'  => self.string_char_literal('"',  TokenType::String),
            '`'  => self.string_char_literal('`',  TokenType::RawString),
            '\'' => self.string_char_literal('\'', TokenType::Char),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            '0' => self.alt_base_number(),

            _ => {
                if is_beginning_digit(c) {
                    self.number(true);
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    self.error("Unexpected character");
                }
            }
        }
    }

    fn get_start_positions(&mut self) {
        self.start_positions.push(0);
        for (i, c) in self.source.chars().enumerate() {
            if c == '\n' {
                self.start_positions.push(i + 1);
            }
        }
    }

    pub fn scan_tokens(&mut self) {
        self.get_start_positions();
        
        while !self.is_at_end() {
            self.start = self.curr;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()), 
            Rc::clone(&self.filename), 
            TokenType::EOF, Rc::from(""), 
            0, 1, self.line
        ));
    }
}