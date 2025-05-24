use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use crate::interpreter::Bytecode;

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

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod test {
    use super::{Keyword, Token, Tokeniser};

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
lbl:
push 1
add
cmp 10
jmp.lt lbl
ret",
                vec![
                    Token::Dot,
                    Token::Keyword(Keyword::Entry),
                    Token::Word("main".into()),
                    Token::Word("main".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value("1".into()),
                    Token::Word("lbl".into()),
                    Token::Colon,
                    Token::Word("push".into()),
                    Token::Value("1".into()),
                    Token::Word("add".into()),
                    Token::Word("cmp".into()),
                    Token::Value("10".into()),
                    Token::Word("jmp".into()),
                    Token::Dot,
                    Token::Word("lt".into()),
                    Token::Word("lbl".into()),
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

pub struct Assembler {
    tokens: Vec<Token>,
    position: usize,
    program: Vec<i64>,
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

    pub fn assemble(mut self) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        self.parse_entry()?;

        while let Some(token) = self.next_token() {
            match token {
                Token::Word(word) => {
                    if self.check_tokens(&[Token::Colon]) {
                        self.labels.insert(word.to_string(), self.program.len());
                        continue;
                    }

                    self.assemble_instruction(word.as_str())?;
                }
                token => Err(format!("unexpected token: {token:?}"))?,
            }
        }

        for (i, label) in self.unresolved {
            let Some(position) = self.labels.get(&label) else {
                Err(format!("could not resolve label: {label}"))?
            };
            self.program[i] = *position as i64;
        }

        Ok(self.program)
    }

    fn parse_entry(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.parse_tokens(&[Token::Dot, Token::Keyword(Keyword::Entry)])?;

        let entry = match self.next_token() {
            Some(Token::Word(entry)) => entry,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        self.program.push(0);
        self.unresolved.insert(0, entry);

        Ok(())
    }

    fn assemble_instruction(&mut self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        match word {
            "push" => self.assemble_operator(Bytecode::Push, 1)?,
            "add" => self.assemble_operator(Bytecode::Add, 0)?,
            "sub" => self.assemble_operator(Bytecode::Sub, 0)?,
            "mul" => self.assemble_operator(Bytecode::Mul, 0)?,
            "div" => self.assemble_operator(Bytecode::Div, 0)?,
            "cmp" => self.assemble_operator(Bytecode::Cmp, 1)?,
            "jmp" => {
                self.assemble_operator(Bytecode::Jmp, 0)?;
                match self.next_token() {
                    Some(Token::Word(label)) => {
                        self.unresolved.insert(self.program.len(), label);
                        self.program.push(0);
                    }
                    Some(Token::Value(value)) => {
                        let value = value.parse()?;
                        self.program.push(value);
                    }
                    Some(token) => Err(format!("unexpected token: {token:?}"))?,
                    _ => unreachable!(),
                }
            }
            "ret" => self.program.push(Bytecode::Ret as i64),
            word => Err(format!("unknown instruction: {word}"))?,
        }

        Ok(())
    }

    fn assemble_operator(
        &mut self,
        code: Bytecode,
        n: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.program.push(code as i64);

        for _ in 0..n {
            let Some(Token::Value(value)) = self.next_token() else {
                Err("expected value for {code:?}")?
            };
            let value = value.parse()?;
            self.program.push(value);
        }

        Ok(())
    }

    fn check_tokens(&mut self, tokens: &[Token]) -> bool {
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

    fn parse_tokens(&mut self, tokens: &[Token]) -> Result<(), Box<dyn std::error::Error>> {
        self.check_tokens(tokens)
            .then_some(())
            .ok_or(format!("expected tokens {tokens:?}").into())
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned()?;
        self.position += 1;
        Some(token)
    }
}
