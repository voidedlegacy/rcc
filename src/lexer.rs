use std::iter;
use std::str;

#[derive(Debug)]
pub enum TokenKind {
    Identifier,
    IntNumber,
    FloatNumber,
    String,
    Char,
    Symbol,
    End,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    val: String,
    line: i32,
}

pub struct Lexer<'a> {
    cur_line: i32,
    filename: String,
    peek: iter::Peekable<str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: String, input: &'a str) -> Lexer<'a> {
        Lexer {
            cur_line: 0,
            filename,
            peek: input.chars().peekable(),
        }
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    pub fn read_token(&mut self) -> Token {
        // TODO: implement proper token reading
        if let Some(&ch) = self.peek.peek() {
            println!("{}", ch);
            self.peek.next();
        }

        Token {
            kind: TokenKind::End,
            val: "".to_string(),
            line: self.cur_line,
        }
    }
}
