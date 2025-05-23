use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum Token {
    Symbol(String),
    Value(String),
    Dot,
    Colon,
    Eof,
}

pub struct TokeniserIter<'t, 's> {
    tokeniser: &'t mut Tokeniser<'s>,
    eof: bool,
}

impl<'t, 's> Iterator for TokeniserIter<'t, 's> {
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

pub struct Tokeniser<'s> {
    src: Peekable<Chars<'s>>,
}

impl<'s> Tokeniser<'s> {
    pub fn new(src: &'s str) -> Self {
        let src = src.chars().peekable();
        Self { src }
    }

    pub fn iter_mut<'t>(&'t mut self) -> TokeniserIter<'t, 's> {
        TokeniserIter {
            tokeniser: self,
            eof: false,
        }
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
                    Token::Symbol(word)
                }
                c => panic!("unexpected char: {c}"),
            },
            None => Token::Eof,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Token, Tokeniser};

    #[test]
    fn test_tokeniser() {
        for (src, want) in [
            ("", vec![Token::Eof]),
            (
                "\n\n; test \tcomment\n\n\nword; test comment",
                vec![Token::Symbol("word".into()), Token::Eof],
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
                    Token::Symbol("entry".into()),
                    Token::Symbol("main".into()),
                    Token::Symbol("main".into()),
                    Token::Colon,
                    Token::Symbol("push".into()),
                    Token::Value("1".into()),
                    Token::Symbol("lbl".into()),
                    Token::Colon,
                    Token::Symbol("push".into()),
                    Token::Value("1".into()),
                    Token::Symbol("add".into()),
                    Token::Symbol("cmp".into()),
                    Token::Value("10".into()),
                    Token::Symbol("jmp".into()),
                    Token::Dot,
                    Token::Symbol("lt".into()),
                    Token::Symbol("lbl".into()),
                    Token::Symbol("ret".into()),
                    Token::Eof,
                ],
            ),
        ] {
            let mut tokeniser = Tokeniser::new(src);
            let have: Vec<Token> = tokeniser.iter_mut().collect();
            assert_eq!(want, have);
        }
    }
}
