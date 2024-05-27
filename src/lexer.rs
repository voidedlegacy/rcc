use std::fs::OpenOptions;
use std::iter;
use std::str;
use std::collections::VecDeque;
use std::path;
use std::process;
use crate::error;
use std::io::Read;

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Identifier,
    IntNumber,
    FloatNumber,
    String,
    Char,
    Symbol,
    Newline,
}

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
            filename,
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

    fn peek_next(&mut self) -> Option<char> {
        if let Some(c) = self.peek_buf.pop_front() {
            Some(c)
        } else {
            self.peek.next()
        }
    }

    fn peek_unget(&mut self, ch: char) {
        self.peek_buf.push_back(ch);
    }

    fn peek_next_char_is(&mut self, ch: char) -> bool {
        if let Some(nextc) = self.peek_next() {
            if nextc == ch {
                return true;
            }
            self.peek_unget(nextc);
        }
        false
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
        if let Some(next) = self.read_token() {
            if next.val == s && next.kind != TokenKind::String && next.kind != TokenKind::Char {
                return true;
            } else {
                self.buf.push_back(next);
            }
        }
        false
    }

    fn unget(&mut self, t: Token) {
        self.buf.push_back(t);
    }

    pub fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.peek_get() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.peek_next();
            } else {
                break;
            }
        }
        Token::new(TokenKind::Identifier, &ident, self.cur_line)
    }

    fn read_number_literal(&mut self) -> Token {
        let mut num = String::new();
        let mut is_float = false;
        while let Some(&c) = self.peek_get() {
            match c {
                '.' | '0'..='9' => {
                    num.push(c);
                    if c == '.' {
                        is_float = true;
                    }
                    self.peek_next();
                }
                _ => break,
            }
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
        let c = self.peek_next().unwrap();
        let mut sym = String::new();
        sym.push(c);
        match c {
            '+' | '-' | '*' | '/' | '%' | '=' | '^' | '!' => {
                if self.peek_char_is('=') {
                    sym.push(self.peek_next().unwrap());
                }
            }
            '<' | '>' | '&' | '|' => {
                if self.peek_char_is(c) {
                    sym.push(self.peek_next().unwrap());
                }
                if self.peek_char_is('=') {
                    sym.push(self.peek_next().unwrap());
                }
            }
            '.' => {
                if self.peek_char_is('.') && self.peek_next_char_is('.') {
                    sym.push(self.peek_next().unwrap());
                    sym.push(self.peek_next().unwrap());
                }
            }
            _ => {}
        }
        Token::new(TokenKind::Symbol, &sym, self.cur_line)
    }

    fn read_string_literal(&mut self) -> Token {
        self.peek_next();
        let mut s = String::new();
        while !self.peek_char_is('\"') {
            s.push(self.peek_next().unwrap());
        }
        self.peek_next();
        Token::new(TokenKind::String, &s, self.cur_line)
    }

    fn read_char_literal(&mut self) -> Token {
        self.peek_next();
        let mut s = String::new();
        while !self.peek_char_is('\'') {
            s.push(self.peek_next().unwrap());
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
            Some(tok) => {
                match tok.kind {
                    TokenKind::Newline => self.read_token(),
                    _ => Some(tok),
                }
            }
            _ => t,
        }
    }

    pub fn get(&mut self) -> Option<Token> {
        let t = self.read_token();
        match t {
            Some(tok) => {
                if tok.val == "#" {
                    // preprocessor directive
                    self.read_cpp_directive();
                    self.get()
                } else {
                    Some(tok)
                }
            }
            _ => t,
        }
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
                filename.push(self.peek_next().unwrap());
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

        if self.skip("(") {
            error::error_exit(self.cur_line, "unsupported");
        }

        println!("macro name: {}", mcro.val);

        let mut body: Vec<Token> = Vec::new();
        print!("macro body: ");
        loop {
            let c = self.do_read_token().unwrap();
            if c.kind == TokenKind::Newline {
                break;
            }
            print!("{}{}", if c.space { " " } else { "" }, c.val);
            body.push(c);
        }
        println!();
    }
}

