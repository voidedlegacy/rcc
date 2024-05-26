use std::collections::VecDeque;
use std::iter;
use std::str;

#[derive(PartialEq)]
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
    pub val: String,
    pub line: i32,
}

impl Token {
    pub fn new(kind: TokenKind, val: &str, line: i32) -> Token {
        Token {
            kind,
            val: val.to_string(),
            line,
        }
    }
}

pub struct Lexer<'a> {
    cur_line: i32,
    filename: String,
    peek: iter::Peekable<str::Chars<'a>>,
    buf: VecDeque<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: String, input: &'a str) -> Lexer<'a> {
        Lexer {
            cur_line: 0,
            filename,
            peek: input.chars().peekable(),
            buf: VecDeque::new(),
        }
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    fn peek_get(&mut self) -> Option<&char> {
        self.buf.front().or_else(|| self.peek.peek())
    }

    fn peek_next(&mut self) -> Option<char> {
        if let Some(c) = self.buf.pop_front() {
            Some(c)
        } else {
            self.peek.next()
        }
    }

    fn peek_unget(&mut self, ch: char) {
        self.buf.push_back(ch);
    }

    fn peek_next_char_is(&mut self, ch: char) -> bool {
        if let Some(nextc) = self.peek_next() {
            self.peek_unget(nextc);
            nextc == ch
        } else {
            false
        }
    }

    fn peek_char_is(&mut self, ch: char) -> bool {
        self.peek_get().map_or(false, |&peekc| peekc == ch)
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
        match self.peek_get() {
            Some(&c) => match c {
                'a'..='z' | 'A'..='Z' | '_' => Some(self.read_identifier()),
                ' ' | '\t' => {
                    self.peek_next();
                    self.read_token()
                }
                '0'..='9' => Some(self.read_number_literal()),
                '\"' => Some(self.read_string_literal()),
                '\'' => Some(self.read_char_literal()),
                '\n' => Some(self.read_newline()),
                _ => Some(self.read_symbol()),
            },
            None => None,
        }
    }

    pub fn read_token(&mut self) -> Option<Token> {
        let token = self.do_read_token();
        if let Some(ref tok) = token {
            if tok.kind == TokenKind::Newline {
                return self.read_token();
            }
        }
        token
    }
}

