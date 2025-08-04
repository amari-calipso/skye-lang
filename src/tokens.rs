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
    Let, While, Enum, Import, Include, Defer,
    Impl, Void, Namespace, Switch, Continue,
    Break, Do, Macro, Const, Use, Try, As,
    Union, Interface, Extern, Super,

    StarConst, RefConst,

    EOF
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Identifier
    }
}

pub type Token = alanglib::token::Token<TokenType>;