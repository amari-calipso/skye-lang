use std::{cmp::max, collections::HashMap, rc::Rc};

use colored::{ColoredString, Colorize};

use crate::{ast::Expression, tokens::Token};

#[macro_export]
macro_rules! get_expect {
    ($obj: expr) => {
        $obj.get().expect(concat!("Could not get Once expression: ", stringify!($obj)))
    };
}

#[macro_export]
macro_rules! dot {
    () => {
        crate::get_expect!(crate::NAMESPACE_SEP)
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedNamedMap<T> {
    pub map: HashMap<Rc<str>, T>,
    pub order: Vec<Rc<str>>
}

impl<T> OrderedNamedMap<T> {
    pub fn new() -> Self {
        Self { 
            map: HashMap::new(), 
            order: Vec::new() 
        }
    }

    pub fn get(&self, k: &str) -> Option<&T> {
        self.map.get(k)
    }

    pub fn contains_key(&self, k: &str) -> bool {
        self.map.contains_key(k)
    }

    pub fn len(&self) -> usize {
        let len = self.order.len();
        debug_assert_eq!(len, self.map.len());
        len
    }

    pub fn insert(&mut self, k: Rc<str>, v: T) -> Option<T> {
        let old = self.map.insert(Rc::clone(&k), v);

        // if the key was already there, removal is O(n). 
        // this shouldn't be used where you expect duplicates (which we don't)
        if old.is_some() {
            self.order.retain(|x| x.as_ref() != k.as_ref());
        }

        self.order.push(k);
        old
    }
}


pub fn substring(string: &String, a: usize, b: usize) -> String {
    string.chars().skip(a).take(b - a).collect()
}

pub fn error_color(msg: &str) -> ColoredString {
    msg.red()
}

pub fn warning_color(msg: &str) -> ColoredString {
    msg.bright_yellow()
}

pub fn note_color(msg: &str) -> ColoredString {
    msg.bright_blue()
}

pub fn report(source: &Rc<str>, msg: &str, type_: &str, filename: &Rc<str>, pos: usize, len: usize, line: usize, color: fn(&str) -> ColoredString) {
    let lines = source.lines().collect::<Vec<&str>>();
    let iter_range = {
        if lines.len() < 5 {
            0..lines.len()
        } else {
            if line <= 2 {
                0..5
            } else if line >= lines.len() - 3 {
                (lines.len() - 5)..lines.len()
            } else {
                (line - 2)..(line + 3)
            }
        }
    };

    let linelen = max((iter_range.end as f64).log10().ceil() as usize, 1);

    println!("{} ({}: line {}, pos {}): {}", color(type_), filename, line + 1, pos, msg);

    for l in iter_range {
        println!("{:linelen$} | {}", l + 1, lines[l].trim_end());

        if l == line {
            println!("{} | {}{}", " ".repeat(linelen), " ".repeat(pos), color("^".repeat(len).as_str()));
        }
    }
}

pub fn error(source: &Rc<str>, msg: &str, filename: &Rc<str>, pos: usize, len: usize, line: usize) {
    report(source, msg, "error", filename, pos, len, line, error_color);
}

pub fn warning(source: &Rc<str>, msg: &str, filename: &Rc<str>, pos: usize, len: usize, line: usize) {
    report(source, msg, "warning", filename, pos, len, line, warning_color);
}

pub fn note(source: &Rc<str>, msg: &str, filename: &Rc<str>, pos: usize, len: usize, line: usize) {
    report(source, msg, "note", filename, pos, len, line, note_color);
}

#[macro_export]
macro_rules! token_error {
    ($slf: expr, $token: expr, $msg: expr) => {
        {
            crate::utils::error(&$token.source, $msg, &$token.filename, $token.pos, $token.end - $token.pos, $token.line);
            $slf.errors += 1;
        }
    };
}

#[macro_export]
macro_rules! token_warning {
    ($token: expr, $msg: expr) => {
        crate::utils::warning(&$token.source, $msg, &$token.filename, $token.pos, $token.end - $token.pos, $token.line);
    };
}

#[macro_export]
macro_rules! token_note {
    ($token: expr, $msg: expr) => {
        crate::utils::note(&$token.source, $msg, &$token.filename, $token.pos, $token.end - $token.pos, $token.line);
    };
}

#[macro_export]
macro_rules! astpos_note {
    ($pos: expr, $msg: expr) => {
        {
            crate::utils::note(&$pos.source, $msg, &$pos.filename, $pos.start, $pos.end - $pos.start, $pos.line);
        }
    };
}


#[macro_export]
macro_rules! ast_error {
    ($slf: expr, $e: expr, $msg: expr) => {
        {
            let pos: crate::ast::AstPos = $e.get_pos();
            crate::utils::error(&pos.source, $msg, &pos.filename, pos.start, pos.end - pos.start, pos.line);
            $slf.errors += 1;
        }
    };
}

#[macro_export]
macro_rules! ast_warning {
    ($s: expr, $msg: expr) => {
        {
            let pos: crate::ast::AstPos = $s.get_pos();
            crate::utils::warning(&pos.source, $msg, &pos.filename, pos.start, pos.end - pos.start, pos.line);
        }
    };
}

#[macro_export]
macro_rules! ast_note {
    ($e: expr, $msg: expr) => {
        {
            let pos: crate::ast::AstPos = $e.get_pos();
            crate::astpos_note!(pos, $msg);
        }
    };
}

pub fn is_beginning_digit(c: char) -> bool {
    c >= '1' && c <= '9'
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn is_bin_digit(c: char) -> bool {
    c == '0' || c == '1'
}

pub fn is_oct_digit(c: char) -> bool {
    c >= '0' && c <= '7'
}

pub fn is_hex_digit(c: char) -> bool {
    is_digit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f')
}

pub fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    c == '_'
}

pub fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

// TODO this actually doesn't work properly,
// it would need to handle variable length escape codes but it doesn't
pub fn get_real_string_length(str: &str) -> usize {
    let mut len: usize = 0;
    let mut backslash = false;

    let mut current_skip = 0;
    let mut target_skip = 0;

    for c in str.chars() {
        if current_skip < target_skip {
            current_skip += 1;
            continue;
        } else {
            target_skip = 0;
        }

        if backslash {
            match c {
                'x' => {
                    current_skip = 0;
                    target_skip = 2;
                    len += 1;
                    continue;
                }
                'u' => {
                    current_skip = 0;
                    target_skip = 4;
                    len += 2;
                    continue;
                }
                'U' => {
                    current_skip = 0;
                    target_skip = 8;
                    len += 4;
                    continue;
                }
                _ => {
                    backslash = false;

                    if is_oct_digit(c) {
                        current_skip = 0;
                        target_skip = 3;
                        continue;
                    } 
                }
            }
        } else {
            if c == '\\' {
                backslash = true;
                continue;
            }
        }

        len += 1
    }

    len
}

pub fn fix_raw_string(str: &str) -> String {
    let mut buf = String::new();

    let mut backslash = false;
    for c in str.chars() {
        if backslash {
            if c == '`' {
                buf.push(c);
            } else {
                buf.push('\\');
                buf.push(c);
            }

            backslash = false;
            continue;
        }

        if c == '"' {
            buf.push('\\');
            buf.push('"');
            continue;
        }

        if c == '\\' {
            backslash = true;
            continue;
        }

        buf.push(c);
    }

    buf
}

pub fn escape_string(str: &str) -> String {
    str.replace('\\', "\\\\")
}

pub fn literal_as_string(expr: Expression) -> Option<(Rc<str>, Token)> {
     match expr {
        Expression::StringLiteral { value, tok, .. } => Some((value, tok)),
        Expression::VoidLiteral(tok) => Some((Rc::from(""), tok)),
        Expression::SignedIntLiteral { value, tok, .. } => Some((value.to_string().into(), tok)),
        Expression::UnsignedIntLiteral { value, tok, .. } => Some((value.to_string().into(), tok)),
        _ => None
    }
}