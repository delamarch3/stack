use std::iter::Peekable;
use std::str::Chars;

use crate::Result;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(String),
    String(String),
    Char(char),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    Keyword(Keyword),
    Value(Value),
    Dot,
    Colon,
    Comma,
    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Entry,
    Data,
    Text,
    Word,
    Dword,
    Byte,
    String,
}

impl<'a> TryFrom<&'a str> for Keyword {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &'a str) -> Result<Self> {
        match value {
            "entry" => Ok(Keyword::Entry),
            "data" => Ok(Keyword::Data),
            "text" => Ok(Keyword::Text),
            "word" => Ok(Keyword::Word),
            "dword" => Ok(Keyword::Dword),
            "byte" => Ok(Keyword::Byte),
            "string" => Ok(Keyword::String),
            _ => Err("not a keyword")?,
        }
    }
}

impl Keyword {
    pub fn is_data_type(&self) -> bool {
        match self {
            Keyword::Word | Keyword::Dword | Keyword::Byte | Keyword::String => true,
            Keyword::Entry | Keyword::Data | Keyword::Text => false,
        }
    }
}

pub struct TokeniserIter<'s> {
    tokeniser: Tokeniser<'s>,
    eof: bool,
}

impl Iterator for TokeniserIter<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            return None;
        }

        let token = self.tokeniser.next_token();
        self.eof = token == Token::Eof;
        Some(token)
    }
}

impl<'a> IntoIterator for Tokeniser<'a> {
    type Item = Token;

    type IntoIter = TokeniserIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokeniserIter {
            tokeniser: self,
            eof: false,
        }
    }
}

pub struct Tokeniser<'s> {
    src: Peekable<Chars<'s>>,
}

impl<'s> Tokeniser<'s> {
    pub fn new(src: &'s str) -> Self {
        let src = src.chars().peekable();
        Self { src }
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> String {
        let mut s = String::new();
        self.extend_while(&mut s, f);
        s
    }

    fn extend_while(&mut self, s: &mut String, f: impl Fn(char) -> bool) {
        while let Some(c) = self.src.peek() {
            if f(*c) {
                s.push(self.src.next().unwrap());
                continue;
            }

            break;
        }
    }

    fn skip_line(&mut self) {
        loop {
            match self.src.peek() {
                Some('\n') => {
                    self.src.next();
                    break;
                }
                Some(_) => {
                    self.src.next();
                }
                None => break,
            }
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        loop {
            match self.src.peek() {
                Some(c) if c.is_whitespace() => {
                    self.src.next();
                    continue;
                }
                Some(';') => {
                    self.skip_line();
                    continue;
                }
                Some(_) => break true,
                None => break false,
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        if !self.skip_whitespace() {
            return Token::Eof;
        }

        match self.src.peek() {
            Some(c) => match c {
                '.' => {
                    self.src.next();
                    Token::Dot
                }
                ',' => {
                    self.src.next();
                    Token::Comma
                }
                ':' => {
                    self.src.next();
                    Token::Colon
                }
                '0'..='9' => {
                    let value = self.take_while(|c| c.is_numeric());
                    Token::Value(Value::Number(value))
                }
                '-' => {
                    let mut value = self.src.next().unwrap().to_string();
                    self.extend_while(&mut value, |c| c.is_numeric());
                    if value == "-" {
                        panic!("unexpected char: -")
                    }
                    Token::Value(Value::Number(value))
                }
                '\'' => {
                    self.src.next();
                    let Some(first) = self.src.next() else {
                        panic!("expected char after '")
                    };

                    let value = match first {
                        '\\' => match self.src.next() {
                            Some(c) => match c {
                                '\\' => '\\',
                                '\'' => '\'',
                                'r' => '\r',
                                't' => '\t',
                                'n' => '\n',
                                '0' => '\0',
                                _ => panic!("unknown character escape"),
                            },
                            None => panic!("expected closing '"),
                        },
                        _ => first,
                    };

                    let Some('\'') = self.src.next() else {
                        panic!("expected closing '")
                    };

                    Token::Value(Value::Char(value))
                }
                '"' => {
                    self.src.next();
                    let value = self.take_while(|c| c != '"');
                    let Some('"') = self.src.next() else {
                        panic!("expected closing \"")
                    };

                    Token::Value(Value::String(value))
                }
                c if c.is_alphabetic() => {
                    let word = self.take_while(|c| c.is_alphanumeric() || ['.', '_'].contains(&c));
                    if let Ok(keyword) = word.as_str().try_into() {
                        Token::Keyword(keyword)
                    } else {
                        Token::Word(word)
                    }
                }
                c => panic!("unexpected char: {c}"),
            },
            None => Token::Eof,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Keyword, Token, Tokeniser, Value};

    #[test]
    fn test_tokeniser() {
        for (src, want) in [
            ("", vec![Token::Eof]),
            (
                "\n\n; test \tcomment\n\n\nword; test comment",
                vec![Token::Keyword(Keyword::Word), Token::Eof],
            ),
            (
                r###"
; My Program
.entry main

.data c .byte '\n'
.data s .string "Hello, World!"
.data n .word -255
.data c2 .word '🤠'

main:
push 1
loop:
push 1
add
cmp 10
jmp.lt loop
ret"###,
                vec![
                    Token::Dot,
                    Token::Keyword(Keyword::Entry),
                    Token::Word("main".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::Data),
                    Token::Word("c".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::Byte),
                    Token::Value(Value::Char('\n')),
                    Token::Dot,
                    Token::Keyword(Keyword::Data),
                    Token::Word("s".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::String),
                    Token::Value(Value::String("Hello, World!".into())),
                    Token::Dot,
                    Token::Keyword(Keyword::Data),
                    Token::Word("n".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::Word),
                    Token::Value(Value::Number("-255".into())),
                    Token::Dot,
                    Token::Keyword(Keyword::Data),
                    Token::Word("c2".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::Word),
                    Token::Value(Value::Char('🤠')),
                    Token::Word("main".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value(Value::Number("1".into())),
                    Token::Word("loop".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value(Value::Number("1".into())),
                    Token::Word("add".into()),
                    Token::Word("cmp".into()),
                    Token::Value(Value::Number("10".into())),
                    Token::Word("jmp.lt".into()),
                    Token::Word("loop".into()),
                    Token::Word("ret".into()),
                    Token::Eof,
                ],
            ),
        ] {
            let have: Vec<Token> = Tokeniser::new(src).into_iter().collect();
            assert_eq!(want, have);
        }
    }
}
