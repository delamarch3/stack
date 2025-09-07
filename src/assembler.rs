use std::collections::HashMap;
use std::mem;

use crate::output::Output;
use crate::program::Bytecode;
use crate::tokeniser::{Keyword, Token, TokenState, Tokeniser, Value};
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
    data: Vec<u8>,
    text: Vec<u8>,
    labels: HashMap<String, Label>,
    unresolved: HashMap<u64, String>,
    macros: HashMap<String, Vec<Token>>,
}

impl Assembler {
    pub fn new() -> Self {
        let data = Vec::new();
        let text = Vec::new();
        let labels = HashMap::new();
        let unresolved = HashMap::new();
        let macros = HashMap::new();

        Self {
            data,
            text,
            labels,
            unresolved,
            macros,
        }
    }

    pub fn assemble(mut self, src: &str) -> Result<Output> {
        let mut tokens = TokenState::new(Tokeniser::new(src).into_iter().collect());

        let entry = self.parse_entry(&mut tokens)?;

        self.assemble_tokens(&mut tokens)?;

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

        let out = Output::new(offset, self.data, self.text, labels);

        Ok(out)
    }

    fn assemble_tokens(&mut self, tokens: &mut TokenState) -> Result<()> {
        while let Some(token) = tokens.next() {
            match token {
                Token::Word(word) => {
                    if tokens.check(&[Token::Colon]) {
                        self.labels
                            .insert(word.to_string(), Label::text(self.text.len()));
                        continue;
                    }

                    self.assemble_instruction(tokens, word.as_str())?;
                }
                Token::Dot => {
                    self.assemble_directive(tokens)?;
                }
                Token::Hash => {
                    self.register_macro(tokens)?;
                }
                Token::At => {
                    self.assemble_expansion(tokens)?;
                }
                Token::Eof => break,
                token => Err(format!("unexpected token: {token:?}"))?,
            }
        }

        Ok(())
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

    fn assemble_directive(&mut self, tokens: &mut TokenState) -> Result<()> {
        match tokens.next_keyword()? {
            Keyword::Data => self.assemble_data(tokens)?,
            keyword => Err(format!("unexpected keyword: {keyword:?}"))?,
        }

        Ok(())
    }

    fn assemble_data(&mut self, tokens: &mut TokenState) -> Result<()> {
        let name = match tokens.next() {
            Some(Token::Word(name)) => name,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            None => todo!(),
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
            tokens.expect(&[Token::Dot])?;
            let size = match tokens.next_keyword()? {
                Keyword::Byte => i8::SIZE,
                Keyword::Word => i32::SIZE,
                Keyword::Dword => i64::SIZE,
                Keyword::String => 0,
                keyword => Err(format!("unexpected keyword: {keyword:?}"))?,
            };

            while {
                match tokens.peek() {
                    Some(Token::Value(value)) => {
                        tokens.next();
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

                tokens.check(&[Token::Comma])
            } {}

            tokens
                .peek_n(1)
                .map(|token| match token {
                    Token::Keyword(keyword) => keyword.is_data_type(),
                    _ => false,
                })
                .unwrap_or_default()
        } {}

        Ok(())
    }

    fn register_macro(&mut self, tokens: &mut TokenState) -> Result<()> {
        let directive = match tokens.next() {
            Some(Token::Word(word)) => word,
            Some(token) => format!("unexpected token: {token:?}"),
            None => todo!(),
        };

        // TODO: keywords?
        match directive.as_str() {
            "define" => {
                let word = match tokens.next() {
                    Some(Token::Word(word)) => word,
                    Some(token) => format!("unexpected token: {token:?}"),
                    None => todo!(),
                };

                tokens.expect(&[Token::LBrace])?;
                let mtokens = tokens.take_while(|token| token != &Token::RBrace);
                tokens.expect(&[Token::RBrace])?;

                self.macros.insert(word, mtokens);
            }
            "include" => todo!(),
            _ => Err(format!("unknown macro directive"))?,
        }

        Ok(())
    }

    fn assemble_expansion(&mut self, tokens: &mut TokenState) -> Result<()> {
        let word = match tokens.next() {
            Some(Token::Word(word)) => word,
            Some(token) => format!("unexpected token: {token:?}"),
            None => todo!(),
        };

        let Some(mut tokens) = self.macros.get(&word).cloned().map(TokenState::new) else {
            Err(format!(
                "macro must be declared before it is expanded: {word}"
            ))?
        };

        self.assemble_tokens(&mut tokens)?;

        Ok(())
    }

    fn assemble_instruction(&mut self, tokens: &mut TokenState, word: &str) -> Result<()> {
        match word {
            "add" | "add.w" => self.assemble_operator(Bytecode::Add),
            "add.b" => self.assemble_operator(Bytecode::AddB),
            "add.d" => self.assemble_operator(Bytecode::AddD),
            "alloc" => self.assemble_operator(Bytecode::Alloc),
            "cmp" | "cmp.w" => self.assemble_operator(Bytecode::Cmp),
            "cmp.d" => self.assemble_operator(Bytecode::CmpD),
            "dataptr" => self.assemble_operator_with_operand::<u64>(tokens, Bytecode::DataPtr)?,
            "div" | "div.w " => self.assemble_operator(Bytecode::Div),
            "div.d" => self.assemble_operator(Bytecode::DivD),
            "dup" | "dup.w" => self.assemble_operator(Bytecode::Dup),
            "dup.d" => self.assemble_operator(Bytecode::DupD),
            "get" | "get.w" => self.assemble_operator(Bytecode::Get),
            "get.b" => self.assemble_operator(Bytecode::GetB),
            "get.d" => self.assemble_operator(Bytecode::GetD),
            "mul" | "mul.w" => self.assemble_operator(Bytecode::Mul),
            "mul.d" => self.assemble_operator(Bytecode::MulD),
            "panic" => self.assemble_operator(Bytecode::Panic),
            "ptr" => self.assemble_operator(Bytecode::Ptr),
            "pop" | "pop.w" => self.assemble_operator(Bytecode::Pop),
            "pop.b" => self.assemble_operator(Bytecode::PopB),
            "pop.d" => self.assemble_operator(Bytecode::PopD),
            "read" => self.assemble_operator(Bytecode::Read),
            "ret" => self.assemble_operator(Bytecode::Ret),
            "ret.d" => self.assemble_operator(Bytecode::RetD),
            "ret.w" => self.assemble_operator(Bytecode::RetW),
            "sub" | "sub.w" => self.assemble_operator(Bytecode::Sub),
            "sub.b" => self.assemble_operator(Bytecode::SubB),
            "sub.d" => self.assemble_operator(Bytecode::SubD),
            "write" => self.assemble_operator(Bytecode::Write),
            "call" => self.assemble_operator_with_label(tokens, Bytecode::Call)?,
            "jmp" => self.assemble_operator_with_label(tokens, Bytecode::Jmp)?,
            "jmp.eq" => self.assemble_operator_with_label(tokens, Bytecode::JmpEq)?,
            "jmp.gt" => self.assemble_operator_with_label(tokens, Bytecode::JmpGt)?,
            "jmp.lt" => self.assemble_operator_with_label(tokens, Bytecode::JmpLt)?,
            "jmp.ne" => self.assemble_operator_with_label(tokens, Bytecode::JmpNe)?,
            "push" | "push.w" => {
                self.assemble_operator_with_operand::<i32>(tokens, Bytecode::Push)?
            }
            "push.d" => self.assemble_operator_with_operand::<i64>(tokens, Bytecode::PushD)?,
            "push.b" => self.assemble_operator_with_operand::<i8>(tokens, Bytecode::PushB)?,
            "load" | "load.w" => {
                self.assemble_operator_with_operand::<u64>(tokens, Bytecode::Load)?
            }
            "load.b" => self.assemble_operator_with_operand::<u64>(tokens, Bytecode::LoadB)?,
            "load.d" => self.assemble_operator_with_operand::<u64>(tokens, Bytecode::LoadD)?,
            "store" | "store.w" => {
                self.assemble_operator_with_operand::<u64>(tokens, Bytecode::Store)?
            }
            "store.b" => self.assemble_operator_with_operand::<u64>(tokens, Bytecode::StoreB)?,
            "store.d" => self.assemble_operator_with_operand::<u64>(tokens, Bytecode::StoreD)?,
            "system" => self.assemble_operator(Bytecode::System),
            word => Err(format!("unknown instruction: {word}"))?,
        }

        Ok(())
    }

    fn assemble_operator(&mut self, code: Bytecode) {
        self.text.push(code as u8);
    }

    fn assemble_operator_with_operand<T>(
        &mut self,
        tokens: &mut TokenState,
        code: Bytecode,
    ) -> Result<()>
    where
        T: Number,
    {
        self.assemble_operator(code);

        match tokens.peek() {
            Some(Token::Value(Value::Number(number))) => {
                tokens.next();
                let value = number
                    .parse::<T>()
                    .map_err(|_| format!("value cannot be parsed: {number}"))?;
                self.text.extend(value.to_le_bytes());
            }
            Some(Token::Word(_)) if T::SIZE == 8 => {
                self.assemble_label(tokens)?;
            }
            Some(Token::At) => {
                tokens.next();

                // TODO: refactor
                let word = match tokens.next() {
                    Some(Token::Word(word)) => word,
                    Some(token) => format!("unexpected token: {token:?}"),
                    None => todo!(),
                };

                let Some(mut mtokens) = self.macros.get(&word).cloned().map(TokenState::new) else {
                    Err(format!(
                        "macro must be declared before it is expanded: {word}"
                    ))?
                };

                match mtokens.next() {
                    Some(Token::Value(Value::Number(number))) => {
                        mtokens.next();
                        let value = number
                            .parse::<T>()
                            .map_err(|_| format!("value cannot be parsed: {number}"))?;
                        self.text.extend(value.to_le_bytes());
                    }
                    Some(Token::Word(_)) if T::SIZE == 8 => {
                        self.assemble_label(&mut mtokens)?;
                    }
                    Some(token) => Err(format!("unexpected token: {token:?}"))?,
                    None => todo!(),
                }

                if let Some(token) = mtokens.next() {
                    Err(format!("unexpected token: {token:?}"))?
                }
            }
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            None => todo!(),
        };

        Ok(())
    }

    fn assemble_operator_with_label(
        &mut self,
        tokens: &mut TokenState,
        code: Bytecode,
    ) -> Result<()> {
        self.assemble_operator(code);
        self.assemble_label(tokens)
    }

    fn assemble_label(&mut self, tokens: &mut TokenState) -> Result<()> {
        match tokens.next() {
            Some(Token::Word(label)) => {
                self.unresolved.insert(self.text.len() as u64, label);
                self.text.extend(0u64.to_le_bytes());
            }
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        Ok(())
    }

    fn parse_entry(&mut self, tokens: &mut TokenState) -> Result<String> {
        tokens.expect(&[Token::Dot, Token::Keyword(Keyword::Entry)])?;

        let entry = match tokens.next() {
            Some(Token::Word(entry)) => entry,
            Some(token) => Err(format!("unexpected token: {token:?}"))?,
            _ => unreachable!(),
        };

        Ok(entry)
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
        let have: Vec<u8> = Assembler::new().assemble(src)?.into();
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
        let have: Vec<u8> = Assembler::new().assemble(src)?.into();
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

#define TWO { 2 }

#define TEST {
    push 1
    push @TWO
    sub
}

main:
    push.d 1
    push.d ptr
    add.d
    push @TWO
    @TEST
    ret
";
        let have: Vec<u8> = Assembler::new().assemble(src)?.into();
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            20, 0, 0, 0, 0, 0, 0, 0,
            9, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::PushD as u8, 1, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::PushD as u8, 12, 0, 0, 0, 0, 0, 0, 0,
            Bytecode::AddD as u8,
            Bytecode::Push as u8, 2, 0, 0, 0,
            Bytecode::Push as u8, 1, 0, 0, 0,
            Bytecode::Push as u8, 2, 0, 0, 0,
            Bytecode::Sub as u8,
            Bytecode::Ret as u8
        ];
        assert_eq!(want, have);
        Ok(())
    }
}
