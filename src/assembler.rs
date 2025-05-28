use std::collections::HashMap;
use std::iter::Peekable;
use std::mem;
use std::str::Chars;

use crate::interpreter::Bytecode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    Keyword(Keyword),
    Value(String),
    Dot,
    Colon,
    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Entry,
}

impl<'a> TryFrom<&'a str> for Keyword {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &'a str) -> Result<Self> {
        match value {
            "entry" => Ok(Keyword::Entry),
            _ => Err("not a keyword")?,
        }
    }
}

pub struct TokeniserIter<'s> {
    tokeniser: Tokeniser<'s>,
    eof: bool,
}

impl<'s> Iterator for TokeniserIter<'s> {
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
        while let Some(c) = self.src.peek() {
            if f(*c) {
                s.push(self.src.next().unwrap());
                continue;
            }

            break;
        }

        s
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
                ':' => {
                    self.src.next();
                    Token::Colon
                }
                '0'..='9' => {
                    let value = self.take_while(|c| c.is_numeric());
                    Token::Value(value)
                }
                c if c.is_alphabetic() => {
                    let word = self.take_while(|c| c.is_alphabetic());
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

pub struct Assembler {
    tokens: Vec<Token>,
    position: usize,
    program: Vec<u8>,
    labels: HashMap<String, usize>,
    unresolved: HashMap<usize, String>,
}

impl Assembler {
    pub fn new(src: &str) -> Self {
        let tokens = Tokeniser::new(src).into_iter().collect();
        let position = 0;
        let program = Vec::new();
        let labels = HashMap::new();
        let unresolved = HashMap::new();

        Self {
            tokens,
            position,
            program,
            labels,
            unresolved,
        }
    }

    pub fn assemble(mut self) -> Result<Vec<u8>> {
        self.parse_entry()?;

        while let Some(token) = self.next_token() {
            match token {
                Token::Word(word) => {
                    if self.check(&[Token::Colon]) {
                        self.labels.insert(word.to_string(), self.program.len());
                        continue;
                    }

                    self.assemble_instruction(word.as_str())?;
                }
                Token::Eof => break,
                token => Err(format!("unexpected token: {token:?}"))?,
            }
        }

        for (i, label) in self.unresolved {
            let Some(position) = self.labels.get(&label) else {
                Err(format!("could not resolve label: {label}"))?
            };
            self.program[i..i + mem::size_of::<i64>()].copy_from_slice(&position.to_be_bytes());
        }

        Ok(self.program)
    }

    fn parse_entry(&mut self) -> Result<()> {
        self.expect(&[Token::Dot, Token::Keyword(Keyword::Entry)])?;

        let entry = match self.next_token() {
            Some(Token::Word(entry)) => entry,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        self.unresolved.insert(self.program.len(), entry);
        self.program.extend(0i64.to_be_bytes());

        Ok(())
    }

    fn assemble_instruction(&mut self, word: &str) -> Result<()> {
        match word {
            "push" => self.assemble_operator(Bytecode::Push, 1)?,
            "pop" => self.assemble_operator(Bytecode::Pop, 0)?,
            "add" => self.assemble_operator(Bytecode::Add, 0)?,
            "sub" => self.assemble_operator(Bytecode::Sub, 0)?,
            "mul" => self.assemble_operator(Bytecode::Mul, 0)?,
            "div" => self.assemble_operator(Bytecode::Div, 0)?,
            "cmp" => self.assemble_operator(Bytecode::Cmp, 1)?,
            "jmp" => {
                let code = if self.check(&[Token::Dot]) {
                    match self.next_token() {
                        Some(Token::Word(word)) => match word.as_str() {
                            "lt" => Bytecode::JmpLt,
                            "gt" => Bytecode::JmpGt,
                            "eq" => Bytecode::JmpEq,
                            "ne" => Bytecode::JmpNe,
                            have => Err(format!("unexpected one of lt, gt, eq, ne. have: {have}"))?,
                        },
                        Some(token) => Err(format!("unexpected token: {token:?}"))?,
                        None => unreachable!(),
                    }
                } else {
                    Bytecode::Jmp
                };

                self.assemble_operator(code, 0)?;
                match self.next_token() {
                    Some(Token::Word(label)) => {
                        self.unresolved.insert(self.program.len(), label);
                        self.program.extend(0i64.to_be_bytes());
                    }
                    Some(Token::Value(value)) => {
                        let value = value.parse::<i64>()?;
                        self.program.extend(value.to_be_bytes());
                    }
                    Some(token) => Err(format!("unexpected token: {token:?}"))?,
                    _ => unreachable!(),
                }
            }
            "swap" => self.assemble_operator(Bytecode::Swap, 0)?,
            "dup" => self.assemble_operator(Bytecode::Dup, 0)?,
            "over" => self.assemble_operator(Bytecode::Over, 0)?,
            "rot" => self.assemble_operator(Bytecode::Rot, 0)?,
            "fail" => self.assemble_operator(Bytecode::Fail, 0)?,
            "ret" => self.assemble_operator(Bytecode::Ret, 0)?,
            word => Err(format!("unknown instruction: {word}"))?,
        }

        Ok(())
    }

    fn assemble_operator(&mut self, code: Bytecode, n: usize) -> Result<()> {
        self.program.extend((code as u8).to_be_bytes());

        for _ in 0..n {
            let Some(Token::Value(value)) = self.next_token() else {
                Err("expected value for {code:?}")?
            };
            let value = value.parse::<i64>()?;
            self.program.extend(value.to_be_bytes());
        }

        Ok(())
    }

    fn check(&mut self, tokens: &[Token]) -> bool {
        let position = self.position;
        for token in tokens {
            if Some(token) == self.next_token().as_ref() {
                continue;
            } else {
                self.position = position;
                return false;
            }
        }

        true
    }

    fn expect(&mut self, tokens: &[Token]) -> Result<()> {
        self.check(tokens)
            .then_some(())
            .ok_or(format!("unexpected token {:?}", self.tokens[self.position]).into())
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned()?;
        self.position += 1;
        Some(token)
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::Bytecode;

    use super::{Assembler, Keyword, Result, Token, Tokeniser};

    #[test]
    fn test_tokeniser() {
        for (src, want) in [
            ("", vec![Token::Eof]),
            (
                "\n\n; test \tcomment\n\n\nword; test comment",
                vec![Token::Word("word".into()), Token::Eof],
            ),
            (
                "
; My Program
.entry main

main:
push 1
loop:
push 1
add
cmp 10
jmp.lt loop
ret",
                vec![
                    Token::Dot,
                    Token::Keyword(Keyword::Entry),
                    Token::Word("main".into()),
                    Token::Word("main".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value("1".into()),
                    Token::Word("loop".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value("1".into()),
                    Token::Word("add".into()),
                    Token::Word("cmp".into()),
                    Token::Value("10".into()),
                    Token::Word("jmp".into()),
                    Token::Dot,
                    Token::Word("lt".into()),
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

    #[test]
    fn test_assemble() -> Result<()> {
        let src = "
; My Program
.entry main

push 1
main:
push 1
loop:
push 1
add
cmp 10
jmp.lt loop
ret";
        let have = Assembler::new(&src).assemble()?;
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 17,
            Bytecode::Push as u8,  0, 0, 0, 0, 0, 0, 0, 1,
            Bytecode::Push as u8,  0, 0, 0, 0, 0, 0, 0, 1, // main:
            Bytecode::Push as u8,  0, 0, 0, 0, 0, 0, 0, 1, // loop:
            Bytecode::Add as u8,
            Bytecode::Cmp as u8,   0, 0, 0, 0, 0, 0, 0, 10,
            Bytecode::JmpLt as u8, 0, 0, 0, 0, 0, 0, 0, 26, // jmp loop
            Bytecode::Ret as u8
        ];
        assert_eq!(want, have);
        Ok(())
    }
}
