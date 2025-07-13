use std::collections::HashMap;
use std::mem;

use crate::output::Output;
use crate::program::Bytecode;
use crate::tokeniser::{Keyword, Token, Tokeniser, Value};
use crate::{Number, Result};

#[derive(PartialEq, Eq)]
enum Section {
    Data,
    Text,
}

#[derive(PartialEq, Eq)]
struct Label {
    section: Section,
    offset: usize,
}

impl Label {
    fn data(offset: usize) -> Self {
        let section = Section::Data;
        Self { section, offset }
    }

    fn text(offset: usize) -> Self {
        let section = Section::Text;
        Self { section, offset }
    }
}

pub struct Assembler {
    tokens: Vec<Token>,
    position: usize,
    data: Vec<u8>,
    text: Vec<u8>,
    labels: HashMap<String, Label>,
    unresolved: HashMap<u64, String>,
}

impl Assembler {
    pub fn new(src: &str) -> Self {
        let tokens = Tokeniser::new(src).into_iter().collect();
        let position = 0;
        let data = Vec::new();
        let text = Vec::new();
        let labels = HashMap::new();
        let unresolved = HashMap::new();

        Self {
            tokens,
            position,
            data,
            text,
            labels,
            unresolved,
        }
    }

    pub fn assemble(mut self) -> Result<Output> {
        let entry = self.parse_entry()?;

        while let Some(token) = self.next_token() {
            match token {
                Token::Word(word) => {
                    if self.check(&[Token::Colon]) {
                        self.labels
                            .insert(word.to_string(), Label::text(self.text.len()));
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

        // Add entry offset to labels
        let mut labels = HashMap::new();
        let offset = self.resolve_label(&entry)? as u64;
        labels.insert(offset, entry);

        // Backpatch and add other offsets to labels
        let unresolved = std::mem::take(&mut self.unresolved);
        for (i, r#ref) in unresolved.into_iter().map(|(k, v)| (k as usize, v)) {
            let offset = self.resolve_label(&r#ref)? as u64;
            self.text[i..i + mem::size_of::<u64>()].copy_from_slice(&offset.to_le_bytes());
            labels.insert(offset, r#ref);
        }

        let out = Output::new(offset as u64, labels, self.data, self.text);

        Ok(out)
    }

    fn resolve_label(&self, r#ref: &str) -> Result<usize> {
        let Some(label) = self.labels.get(r#ref) else {
            Err(format!("could not resolve label: {}", r#ref))?
        };

        // Since the program is loaded as [entry][data][text], the data section offsets stay as is
        // while the text offsets are offset further by the data length
        let offset = match label.section {
            Section::Data => mem::size_of::<u64>() + label.offset,
            Section::Text => mem::size_of::<u64>() + label.offset + self.data.len(),
        };

        Ok(offset)
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

        let offset = self.data.len();
        if self
            .labels
            .insert(name.clone(), Label::data(offset))
            .is_some()
        {
            Err(format!("label is declared twice: {name}"))?;
        }

        while {
            self.expect(&[Token::Dot])?;
            let size = match self.next_keyword()? {
                Keyword::Byte => i8::SIZE,
                Keyword::Word => i32::SIZE,
                Keyword::Dword => i64::SIZE,
                Keyword::String => 0,
                keyword => Err(format!("unexpected keyword: {keyword:?}"))?,
            };

            while {
                match self.peek_token() {
                    Some(Token::Value(value)) => {
                        self.next_token();
                        match value {
                            Value::Number(number) if size == i8::SIZE => {
                                let value = number.parse::<i8>()?;
                                self.data.extend(value.to_le_bytes());
                            }
                            Value::Number(number) if size == i32::SIZE => {
                                let value = number.parse::<i32>()?;
                                self.data.extend(value.to_le_bytes());
                            }
                            Value::Number(number) if size == i64::SIZE => {
                                let value = number.parse::<i64>()?;
                                self.data.extend(value.to_le_bytes());
                            }
                            Value::Char(char) if size == i8::SIZE && char.is_ascii() => {
                                let value: u8 = char.try_into().unwrap();
                                self.data.extend(value.to_le_bytes());
                            }
                            Value::Char(char) if size == i32::SIZE => {
                                let value = char as u32;
                                self.data.extend(value.to_le_bytes());
                            }
                            Value::String(string) if size == 0 => {
                                self.data.extend(string.into_bytes());
                            }
                            value => Err(format!("value {value:?} does not match size {size}"))?,
                        }
                    }
                    _ => self.data.extend(std::iter::repeat_n(0u8, size)),
                };

                self.check(&[Token::Comma])
            } {}

            self.peek_n_token(1)
                .map(|token| match token {
                    Token::Keyword(keyword) => keyword.is_data_type(),
                    _ => false,
                })
                .unwrap_or_default()
        } {}

        Ok(())
    }

    fn assemble_instruction(&mut self, word: &str) -> Result<()> {
        match word {
            "push" | "push.w" => self.assemble_operator_with_operand::<i32>(Bytecode::Push)?,
            "push.d" => self.assemble_operator_with_operand::<i64>(Bytecode::PushD)?,
            "push.b" => self.assemble_operator_with_operand::<i8>(Bytecode::PushB)?,
            "pop" | "pop.w" => self.assemble_operator(Bytecode::Pop),
            "pop.d" => self.assemble_operator(Bytecode::PopD),
            "pop.b" => self.assemble_operator(Bytecode::PopB),
            "load" | "load.w" => self.assemble_operator_with_operand::<u64>(Bytecode::Load)?,
            "load.d" => self.assemble_operator_with_operand::<u64>(Bytecode::LoadD)?,
            "load.b" => self.assemble_operator_with_operand::<u64>(Bytecode::LoadB)?,
            "store" | "store.w" => self.assemble_operator_with_operand::<u64>(Bytecode::Store)?,
            "store.d" => self.assemble_operator_with_operand::<u64>(Bytecode::StoreD)?,
            "store.b" => self.assemble_operator_with_operand::<u64>(Bytecode::StoreB)?,
            "get" | "get.w" => self.assemble_operator(Bytecode::Get),
            "get.d" => self.assemble_operator(Bytecode::GetD),
            "get.b" => self.assemble_operator(Bytecode::GetB),
            "add" | "add.w" => self.assemble_operator(Bytecode::Add),
            "add.d" => self.assemble_operator(Bytecode::AddD),
            "add.b" => self.assemble_operator(Bytecode::AddB),
            "sub" | "sub.w" => self.assemble_operator(Bytecode::Sub),
            "sub.d" => self.assemble_operator(Bytecode::SubD),
            "sub.b" => self.assemble_operator(Bytecode::SubB),
            "mul" | "mul.w" => self.assemble_operator(Bytecode::Mul),
            "mul.d" => self.assemble_operator(Bytecode::MulD),
            "div" | "div.w " => self.assemble_operator(Bytecode::Div),
            "div.d" => self.assemble_operator(Bytecode::DivD),
            "cmp" | "cmp.w" => self.assemble_operator(Bytecode::Cmp),
            "cmp.d" => self.assemble_operator(Bytecode::CmpD),
            "dup" | "dup.w" => self.assemble_operator(Bytecode::Dup),
            "dup.d" => self.assemble_operator(Bytecode::DupD),
            "fail" => self.assemble_operator(Bytecode::Fail),
            "ret" => self.assemble_operator(Bytecode::Ret),
            "ret.w" => self.assemble_operator(Bytecode::RetW),
            "ret.d" => self.assemble_operator(Bytecode::RetD),
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
        self.text.extend(std::iter::once(code as u8));
    }

    fn assemble_operator_with_operand<T>(&mut self, code: Bytecode) -> Result<()>
    where
        T: Number,
    {
        self.assemble_operator(code);

        match self.peek_token() {
            Some(Token::Value(Value::Number(number))) => {
                self.next_token();
                let value = number
                    .parse::<T>()
                    .map_err(|_| format!("value cannot be parsed: {number}"))?;
                self.text.extend(value.to_le_bytes());
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
                self.unresolved.insert(self.text.len() as u64, label);
                self.text.extend(0u64.to_le_bytes());
            }
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        Ok(())
    }

    fn parse_entry(&mut self) -> Result<String> {
        self.expect(&[Token::Dot, Token::Keyword(Keyword::Entry)])?;

        let entry = match self.next_token() {
            Some(Token::Word(entry)) => entry,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        Ok(entry)
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

    fn peek_n_token(&self, n: usize) -> Option<Token> {
        let position = self.position + n;
        self.tokens.get(position).cloned()
    }

    fn peek_token(&self) -> Option<Token> {
        self.peek_n_token(0)
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
    use crate::program::Bytecode;
    use crate::Result;

    use super::Assembler;

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
push 10
cmp
jmp.lt loop
ret";
        let have: Vec<u8> = Assembler::new(&src).assemble()?.into();
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            13, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::Push as u8, 1, 0, 0, 0,
            Bytecode::Push as u8, 1, 0, 0, 0, // main:
            Bytecode::Push as u8, 1, 0, 0, 0, // loop:
            Bytecode::Add as u8,
            Bytecode::Push as u8, 10, 0, 0, 0,
            Bytecode::Cmp as u8,
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
        let have: Vec<u8> = Assembler::new(&src).assemble()?.into();
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
        let have: Vec<u8> = Assembler::new(&src).assemble()?.into();
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
