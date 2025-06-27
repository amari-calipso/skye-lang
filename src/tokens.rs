use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftSquare, RightSquare,

    Comma, Dot, Minus, Plus, Semicolon,
    Slash, Star, Colon, ColonColon, At,
    ShiftLeft, ShiftRight, Mod, Tilde,
    Arrow, Hash, DotDot,

    PlusPlus, MinusMinus,
    PlusEquals, MinusEquals,
    StarEquals, SlashEquals,
    OrEquals, AndEquals,
    XorEquals, ModEquals,
    ShiftLeftEquals, ShiftRightEquals,

    Bang, Question, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    LogicAnd, LogicOr,
    BitwiseAnd, BitwiseOr, BitwiseXor,

    Identifier, RawString, String, Char,
    U8, U16, U32, U64, Usz,
    I8, I16, I32, I64, AnyInt,
    F32, F64, AnyFloat,

    Struct, Else, Fn, For, If, Return,
    Let, While, Enum, Import, Defer,
    Impl, Void, Namespace, Switch, Continue,
    Break, Do, Macro, Const, Use, Try, As,
    Default, Union, Interface,

    StarConst, RefConst, Reserved,

    EOF
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Eq)]
pub struct Token {
    pub source: Rc<str>,
    pub filename: Rc<str>,
    pub type_: TokenType,
    pub lexeme: Rc<str>,
    pub pos: usize,
    pub end: usize,
    pub line: usize
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").field("type_", &self.type_).field("lexeme", &self.lexeme).field("pos", &self.pos).field("end", &self.end).field("line", &self.line).finish()
    }
}

impl Token {
    pub fn new(source: Rc<str>, filename: Rc<str>, type_: TokenType, lexeme: Rc<str>, pos: usize, end: usize, line: usize) -> Self {
        Token { source, filename, type_, lexeme, pos, end, line }
    }

    pub fn dummy(lexeme: Rc<str>) -> Self {
        Token { source: Rc::from(""), filename: Rc::from(""), type_: TokenType::Identifier, lexeme, pos: 0, end: 1, line: 0 }
    }

    pub fn empty() -> Self {
        Token::dummy(Rc::from(""))
    }

    pub fn set_type(&mut self, type_: TokenType) {
        self.type_ = type_;
    }

    pub fn set_lexeme(&mut self, lexeme: &str) {
        self.lexeme = Rc::from(lexeme);
    }
}