use std::collections::HashMap;
use std::iter::Peekable;
use std::mem;
use std::str::{Chars, FromStr};

use crate::interpreter::Bytecode;
use crate::Number;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Number(String),
    String(String),
    Char(char),
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Word(String),
    Keyword(Keyword),
    Value(Value),
    Dot,
    Colon,
    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Keyword {
    Entry,
    Data,
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
            "word" => Ok(Keyword::Word),
            "dword" => Ok(Keyword::Dword),
            "byte" => Ok(Keyword::Byte),
            "string" => Ok(Keyword::String),
            _ => Err("not a keyword")?,
        }
    }
}

struct TokeniserIter<'s> {
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

struct Tokeniser<'s> {
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
                    let Some(value) = self.src.next() else {
                        panic!("expected char after '")
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
                    let word = self.take_while(|c| c.is_alphanumeric() || c == '.');
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
        self.expect(&[Token::Dot, Token::Keyword(Keyword::Entry)])?;
        self.assemble_label()?;

        while let Some(token) = self.next_token() {
            match token {
                Token::Word(word) => {
                    if self.check(&[Token::Colon]) {
                        self.labels.insert(word.to_string(), self.program.len());
                        continue;
                    }

                    self.assemble_instruction(word.as_str())?;
                }
                Token::Dot => {
                    self.assemble_directive()?;
                }
                Token::Eof => break,
                token => Err(format!("unexpected token: {token:?}"))?,
            }
        }

        for (i, label) in self.unresolved {
            let Some(offset) = self.labels.get(&label) else {
                Err(format!("could not resolve label: {label}"))?
            };
            self.program[i..i + mem::size_of::<u64>()].copy_from_slice(&offset.to_le_bytes());
        }

        Ok(self.program)
    }

    fn assemble_directive(&mut self) -> Result<()> {
        match self.next_keyword()? {
            Keyword::Data => self.assemble_data()?,
            keyword => Err(format!("unexpected keyword: {keyword:?}"))?,
        }

        Ok(())
    }

    fn assemble_data(&mut self) -> Result<()> {
        let name = match self.next_token() {
            Some(Token::Word(name)) => name,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            None => unreachable!(),
        };

        let offset = self.program.len();
        if self.labels.insert(name.clone(), offset).is_some() {
            Err(format!("label is declared twice: {name}"))?;
        }

        // TODO: parse records
        self.expect(&[Token::Dot])?;
        let size = match self.next_keyword()? {
            Keyword::Byte => i8::SIZE,
            Keyword::Word => i32::SIZE,
            Keyword::Dword => i64::SIZE,
            Keyword::String => 0,
            keyword => Err(format!("unexpected keyword: {keyword:?}"))?,
        };

        match self.peek_token() {
            Some(Token::Value(value)) => {
                self.next_token();
                match value {
                    Value::Number(number) if size == i8::SIZE => {
                        let value = number.parse::<i8>()?;
                        self.program.extend(&value.to_le_bytes());
                    }
                    Value::Number(number) if size == i32::SIZE => {
                        let value = number.parse::<i32>()?;
                        self.program.extend(&value.to_le_bytes());
                    }
                    Value::Number(number) if size == i64::SIZE => {
                        let value = number.parse::<i64>()?;
                        self.program.extend(&value.to_le_bytes());
                    }
                    Value::Char(char) if size == i8::SIZE && char.is_ascii() => {
                        let value: u8 = char.try_into().unwrap();
                        self.program.extend(&value.to_le_bytes());
                    }
                    Value::Char(char) if size == i32::SIZE => {
                        let value: u32 = char.try_into().unwrap();
                        self.program.extend(&value.to_le_bytes());
                    }
                    Value::String(string) if size == 0 => {
                        self.program.extend(string.into_bytes());
                    }
                    value => Err(format!("value {value:?} does not match size {size}"))?,
                }
            }
            _ => self.program.extend(std::iter::repeat_n(0u8, size)),
        };

        Ok(())
    }

    fn assemble_instruction(&mut self, word: &str) -> Result<()> {
        match word {
            "push" => self.assemble_operator_with_operand::<i32>(Bytecode::Push)?,
            "push.d" => self.assemble_operator_with_operand::<i64>(Bytecode::PushD)?,
            "push.b" => self.assemble_operator_with_operand::<i8>(Bytecode::PushB)?,
            "pop" => self.assemble_operator(Bytecode::Pop),
            "pop.d" => self.assemble_operator(Bytecode::PopD),
            "pop.b" => self.assemble_operator(Bytecode::PopB),
            "load" => self.assemble_operator_with_operand::<usize>(Bytecode::Load)?,
            "load.d" => self.assemble_operator_with_operand::<usize>(Bytecode::LoadD)?,
            "load.b" => self.assemble_operator_with_operand::<usize>(Bytecode::LoadB)?,
            "store" => self.assemble_operator_with_operand::<usize>(Bytecode::Store)?,
            "store.d" => self.assemble_operator_with_operand::<usize>(Bytecode::StoreD)?,
            "store.b" => self.assemble_operator_with_operand::<usize>(Bytecode::StoreB)?,
            "get" => self.assemble_operator_with_operand::<usize>(Bytecode::Get)?,
            "get.d" => self.assemble_operator_with_operand::<usize>(Bytecode::GetD)?,
            "get.b" => self.assemble_operator_with_operand::<usize>(Bytecode::GetB)?,
            "add" => self.assemble_operator(Bytecode::Add),
            "add.d" => self.assemble_operator(Bytecode::AddD),
            "add.b" => self.assemble_operator(Bytecode::AddB),
            "sub" => self.assemble_operator(Bytecode::Sub),
            "sub.d" => self.assemble_operator(Bytecode::SubD),
            "sub.b" => self.assemble_operator(Bytecode::SubB),
            "mul" => self.assemble_operator(Bytecode::Mul),
            "mul.d" => self.assemble_operator(Bytecode::MulD),
            "div" => self.assemble_operator(Bytecode::Div),
            "div.d" => self.assemble_operator(Bytecode::DivD),
            "cmp" => self.assemble_operator_with_operand::<i32>(Bytecode::Cmp)?,
            "cmp.d" => self.assemble_operator_with_operand::<i64>(Bytecode::CmpD)?,
            "dup" => self.assemble_operator(Bytecode::Dup),
            "dup.d" => self.assemble_operator(Bytecode::DupD),
            "fail" => self.assemble_operator(Bytecode::Fail),
            "ret" => self.assemble_operator(Bytecode::Ret),
            "ret.d" => todo!(),
            "ret.b" => todo!(),
            "call" => self.assemble_operator_with_label(Bytecode::Call)?,
            "jmp" => self.assemble_operator_with_label(Bytecode::Jmp)?,
            "jmp.lt" => self.assemble_operator_with_label(Bytecode::JmpLt)?,
            "jmp.gt" => self.assemble_operator_with_label(Bytecode::JmpGt)?,
            "jmp.eq" => self.assemble_operator_with_label(Bytecode::JmpEq)?,
            "jmp.ne" => self.assemble_operator_with_label(Bytecode::JmpNe)?,
            word => Err(format!("unknown instruction: {word}"))?,
        }

        Ok(())
    }

    fn assemble_operator(&mut self, code: Bytecode) {
        self.program.extend((code as u8).to_le_bytes());
    }

    fn assemble_operator_with_operand<T>(&mut self, code: Bytecode) -> Result<()>
    where
        T: FromStr + Number,
    {
        self.assemble_operator(code);

        match self.peek_token() {
            Some(Token::Value(Value::Number(number))) => {
                self.next_token();
                let value = number
                    .parse::<T>()
                    .map_err(|_| format!("value cannot be parsed: {number}"))?;
                self.program.extend(value.to_le_bytes());
            }
            Some(Token::Word(_)) if T::SIZE == 8 => {
                self.assemble_label()?;
            }
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            None => unreachable!(),
        };

        Ok(())
    }

    fn assemble_operator_with_label(&mut self, code: Bytecode) -> Result<()> {
        self.assemble_operator(code);
        self.assemble_label()
    }

    fn assemble_label(&mut self) -> Result<()> {
        match self.next_token() {
            Some(Token::Word(label)) => {
                self.unresolved.insert(self.program.len(), label);
                self.program.extend(0u64.to_le_bytes());
            }
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

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

    fn peek_token(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned()?;
        Some(token)
    }

    fn next_keyword(&mut self) -> Result<Keyword> {
        match self.next_token() {
            Some(Token::Keyword(keyword)) => Ok(keyword),
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::Bytecode;

    use super::{Assembler, Keyword, Result, Token, Tokeniser, Value};

    #[test]
    fn test_tokeniser() {
        for (src, want) in [
            ("", vec![Token::Eof]),
            (
                "\n\n; test \tcomment\n\n\nword; test comment",
                vec![Token::Keyword(Keyword::Word), Token::Eof],
            ),
            (
                "
; My Program
.entry main

.data c .byte 'a'
.data s .string \"Hello, World!\"
.data n .word -255
.data c2 .word '🤠'

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
                    Token::Dot,
                    Token::Keyword(Keyword::Data),
                    Token::Word("c".into()),
                    Token::Dot,
                    Token::Keyword(Keyword::Byte),
                    Token::Value(Value::Char('a')),
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
            13, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Push as u8,  1, 0, 0, 0,
            Bytecode::Push as u8,  1, 0, 0, 0, // main:
            Bytecode::Push as u8,  1, 0, 0, 0, // loop:
            Bytecode::Add as u8,
            Bytecode::Cmp as u8,   10, 0, 0, 0,
            Bytecode::JmpLt as u8, 18, 0, 0, 0, 0, 0, 0, 0, // jmp loop
            Bytecode::Ret as u8
        ];
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_assemble2() -> Result<()> {
        let src = "
.entry main

main:
    push 22
    push 33
    call add ; local0 = 22, local1 = 33
    store 0
    ret

add:
   load 0
   load 1
   add
   ret";
        let have = Assembler::new(&src).assemble()?;
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            8, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Push as u8, 22, 0, 0, 0,
            Bytecode::Push as u8, 33, 0, 0, 0,
            Bytecode::Call as u8, 37, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Store as u8, 0, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Ret as u8,
            Bytecode::Load as u8, 0, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Load as u8, 1, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Add as u8,
            Bytecode::Ret as u8
        ];
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_assemble3() -> Result<()> {
        let src = "
.entry main

.data input .word 9
.data ptr .dword

main:
    push.d 1
    push.d ptr
    add.d
    ret
";
        let have = Assembler::new(&src).assemble()?;
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            20, 0, 0, 0, 0, 0, 0, 0,
            9, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::PushD as u8, 1, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::PushD as u8, 12, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::AddD as u8,
            Bytecode::Ret as u8
        ];
        assert_eq!(want, have);
        Ok(())
    }
}
