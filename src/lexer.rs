use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::iter;
use std::str;
use std::collections::VecDeque;
use std::path;
use std::process;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error;

lazy_static! {
    static ref MacroMap: Arc<Mutex<HashMap<String, Macro>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub enum Macro {
    Object(Vec<Token>),
    // FuncLile()
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    Identifier,
    IntNumber,
    FloatNumber,
    String,
    Char,
    Symbol,
    Newline,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub space: bool, // leading space
    pub val: String,
    pub line: i32,
}

impl Token {
    pub fn new(kind: TokenKind, val: &str, line: i32) -> Token {
        Token {
            kind,
            space: false,
            val: val.to_string(),
            line,
        }
    }
}

pub struct Lexer<'a> {
    cur_line: i32,
    filename: String,
    peek: iter::Peekable<str::Chars<'a>>,
    peek_buf: VecDeque<char>,
    buf: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: String, input: &'a str) -> Lexer<'a> {
        Lexer {
            cur_line: 1,
            filename: filename.to_string(),
            peek: input.chars().peekable(),
            peek_buf: VecDeque::new(),
            buf: VecDeque::new(),
        }
    }

    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }

    fn peek_get(&mut self) -> Option<&char> {
        self.peek_buf.front().or_else(|| self.peek.peek())
    }

    fn peek_next(&mut self) -> char {
        if let Some(c) = self.peek_buf.pop_front() {
            c
        } else {
            self.peek.next().unwrap()
        }
    }

    fn peek_unget(&mut self, ch: char) {
        self.peek_buf.push_back(ch);
    }

    fn peek_next_char_is(&mut self, ch: char) -> bool {
        let c = self.peek_next();
        let nextc = self.peek_next();
        self.peek_unget(c);
        self.peek_unget(nextc);
        nextc == ch
    }

    fn peek_char_is(&mut self, ch: char) -> bool {
        let line = self.cur_line;
        let errf = || -> Option<&char> {
            error::error_exit(line, format!("expected '{}'", ch).as_str());
            None
        };

        let peekc = self.peek_get().or_else(errf).unwrap();
        *peekc == ch
    }

    fn skip(&mut self, s: &str) -> bool {
        let next = self.read_token();
        let n = next.ok_or("error").unwrap();
        if n.val == s && n.kind != TokenKind::String && n.kind != TokenKind::Char {
            true
        } else {
            self.buf.push_back(n);
            false
        }
    }

    fn unget(&mut self, t: Token) {
        self.buf.push_back(t);
    }

    pub fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        loop {
            match self.peek_get() {
                Some(&c) => match c {
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => ident.push(c),
                    _ => break,
                },
                _ => break,
            };
            self.peek_next();
        }
        Token::new(TokenKind::Identifier, &ident, self.cur_line)
    }

    fn read_number_literal(&mut self) -> Token {
        let mut num = String::new();
        let mut is_float = false;
        loop {
            match self.peek_get() {
                Some(&c) => match c {
                    '.' | '0'..='9' => {
                        num.push(c);
                        if c == '.' {
                            is_float = true;
                        }
                    }
                    _ => break,
                },
                _ => break,
            };
            self.peek_next();
        }
        if is_float {
            Token::new(TokenKind::FloatNumber, &num, self.cur_line)
        } else {
            Token::new(TokenKind::IntNumber, &num, self.cur_line)
        }
    }

    pub fn read_newline(&mut self) -> Token {
        self.peek_next();
        self.cur_line += 1;
        Token::new(TokenKind::Newline, "", self.cur_line)
    }

    pub fn read_symbol(&mut self) -> Token {
        let c = self.peek_next();
        let mut sym = String::new();
        sym.push(c);
        match c {
            '+' | '-' | '*' | '/' | '%' | '=' | '^' | '!' => {
                if self.peek_char_is('=') {
                    sym.push(self.peek_next());
                }
            }
            '<' | '>' | '&' | '|' => {
                if self.peek_char_is(c) {
                    sym.push(self.peek_next());
                }
                if self.peek_char_is('=') {
                    sym.push(self.peek_next());
                }
            }
            '.' => {
                if self.peek_char_is('.') && self.peek_next_char_is('.') {
                    sym.push(self.peek_next());
                    sym.push(self.peek_next());
                }
            }
            _ => {}
        };
        Token::new(TokenKind::Symbol, &sym, self.cur_line)
    }

    fn read_string_literal(&mut self) -> Token {
        self.peek_next();
        let mut s = String::new();
        while !self.peek_char_is('\"') {
            s.push(self.peek_next());
        }
        self.peek_next();
        Token::new(TokenKind::String, &s, self.cur_line)
    }

    fn read_char_literal(&mut self) -> Token {
        self.peek_next();
        let mut s = String::new();
        while !self.peek_char_is('\'') {
            s.push(self.peek_next());
        }
        self.peek_next();
        Token::new(TokenKind::Char, &s, self.cur_line)
    }

    pub fn do_read_token(&mut self) -> Option<Token> {
        if !self.buf.is_empty() {
            return self.buf.pop_front();
        }

        match self.peek_get() {
            Some(&c) => match c {
                'a'..='z' | 'A'..='Z' | '_' => Some(self.read_identifier()),
                ' ' | '\t' => {
                    self.peek_next();
                    // set a leading space
                    fn f(tok: Token) -> Option<Token> {
                        let mut t = tok;
                        t.space = true;
                        Some(t)
                    }
                    self.read_token().and_then(f)
                }
                '0'..='9' => Some(self.read_number_literal()),
                '\"' => Some(self.read_string_literal()),
                '\'' => Some(self.read_char_literal()),
                '\n' => Some(self.read_newline()),
                '\\' => {
                    self.peek_next();
                    self.read_token()
                }
                '/' => {
                    if self.peek_next_char_is('*') {
                        self.peek_next(); // /
                        self.peek_next(); // *
                        while !(self.peek_char_is('*') && self.peek_next_char_is('/')) {
                            self.peek_next();
                        }
                        self.peek_next();
                        self.peek_next();
                        self.do_read_token()
                    } else if self.peek_next_char_is('/') {
                        self.peek_next(); // /
                        self.peek_next(); // /
                        while !self.peek_char_is('\n') {
                            self.peek_next();
                        }
                        self.do_read_token()
                    } else {
                        Some(self.read_symbol())
                    }
                }
                _ => Some(self.read_symbol()),
            },
            None => None,
        }
    }

    pub fn read_token(&mut self) -> Option<Token> {
        let t = self.do_read_token();
        match t {
            Some(tok) => match tok.kind {
                TokenKind::Newline => self.read_token(),
                _ => Some(tok),
            },
            _ => t,
        }
    }

    fn expand(&mut self, token: Option<Token>) -> Option<Token> {
        token.and_then(|tok| match MacroMap.lock().unwrap().get(tok.val.as_str()) {
            Some(a) => match a {
                Macro::Object(ref t) => {
                    for tt in t.iter().rev() {
                        self.unget(tt.clone());
                    }
                    self.read_token()
                }
            },
            None => Some(tok),
        })
    }

    pub fn get(&mut self) -> Option<Token> {
        let t = self.read_token();
        let tok = match t {
            Some(tok) => {
                if tok.val == "#" {
                    // preprocessor directive
                    self.read_cpp_directive();
                    self.get()
                } else {
                    Some(tok)
                }
            }
            _ => return t,
        };
        self.expand(tok)
    }

    // for c preprocessor

    fn read_cpp_directive(&mut self) {
        let t = self.do_read_token(); // cpp directive
        match t.ok_or("error").unwrap().val.as_str() {
            "include" => self.read_cpp_include(),
            "define" => self.read_cpp_define(),
            _ => {}
        }
    }

    fn cpp_try_include(&mut self, filename: &str) -> Option<String> {
        let header_paths = vec![
            "./include/",
            "/include/",
            "/usr/include/",
            "/usr/include/linux/",
            "/usr/include/x86_64-linux-gnu/",
            "",
        ];
        for header_path in header_paths {
            let real_filename = format!("{}{}", header_path, filename);
            if path::Path::new(&real_filename).exists() {
                return Some(real_filename);
            }
        }
        None
    }

    fn read_cpp_include(&mut self) {
        let mut filename = String::new();
        if self.skip("<") {
            while !self.peek_char_is('>') {
                filename.push(self.peek_next());
            }
            self.peek_next();
        }
        let real_filename = match self.cpp_try_include(&filename) {
            Some(f) => f,
            _ => {
                eprintln!("error: {}: not found '{}'", self.cur_line, filename);
                process::exit(-1)
            }
        };
        println!("include filename: {}", real_filename);
        let mut file = OpenOptions::new().read(true).open(&real_filename).unwrap();
        let mut body = String::new();
        file.read_to_string(&mut body).unwrap();
        let mut lexer = Lexer::new(filename.clone(), &body);
        while let Some(tok) = lexer.get() {
            self.buf.push_back(tok);
        }
        println!("end of: {}", real_filename);
    }

    fn read_cpp_define(&mut self) {
        let mcro = self.do_read_token().unwrap();
        assert_eq!(mcro.kind, TokenKind::Identifier);

        // TODO: func like macro is unsupported now..
        if self.skip("(") {
            error::error_exit(self.cur_line, "unsupported");
        }

        println!("\tmacro name: {}", mcro.val);

        let mut body: Vec<Token> = Vec::new();
        print!("\tmacro body: ");
        loop {
            let c = self.do_read_token().unwrap();
            if c.kind == TokenKind::Newline {
                break;
            }
            print!("{}{}", if c.space { " " } else { "" }, c.val);
            body.push(c);
        }
        println!();
        MacroMap.lock().unwrap().insert(mcro.val, Macro::Object(body));
    }
}
