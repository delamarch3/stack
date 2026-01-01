use std::{
    fs::File,
    io::Read,
    iter::Peekable,
    path::PathBuf,
    str::{Chars, Lines},
    sync::{Arc, Mutex},
};

use stack::{assembler::Assembler, interpreter::Interpreter, SharedWriter};

const SEPARATOR: &str = "----";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct AssertionError {
    filename: &'static str,
    testname: String,
    message: String,
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: assertion error: {}",
            self.filename, self.testname, self.message
        )
    }
}

pub struct TestRunner {
    filename: &'static str,
    include_paths: Vec<PathBuf>,
    errors: Vec<AssertionError>,
}

impl TestRunner {
    pub fn new(filename: &'static str, include_paths: Vec<PathBuf>) -> Self {
        Self {
            filename,
            include_paths,
            errors: Vec::new(),
        }
    }

    pub fn run(mut self, testcases: Vec<TestCase>) -> Result<Vec<AssertionError>> {
        for testcase in testcases {
            self.run_one(testcase)?
        }

        Ok(self.errors)
    }

    fn run_one(&mut self, testcase: TestCase) -> Result<()> {
        let output = Assembler::new()
            .with_include_paths(self.include_paths.clone())
            .assemble(&testcase.src)?;

        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = None;
        // TODO: this could panic, which we should interpret as an error (or new panic status?)
        let mut interpreter =
            Interpreter::new(&output, Some(Arc::clone(&stdout) as SharedWriter), stderr)?;

        let status = if interpreter.run().is_ok() {
            Status::Ok
        } else {
            Status::Error
        };

        let stack = interpreter.frames().last().unwrap().opstack.as_slice();

        if testcase.status != status {
            self.add_error(
                &testcase,
                format!("status mismatch: want {}, have {}", testcase.status, status),
            );
        }

        if let Some(want) = &testcase.stack {
            let want = want.as_slice();

            let have = unsafe {
                let (prefix, have, suffix) = stack.align_to::<i32>();

                // stack is aligned to 8 bytes, so these should always be empty
                assert!(prefix.is_empty());
                assert!(suffix.is_empty());
                have
            };

            if want != have {
                self.add_error(
                    &testcase,
                    format!("stack mismatch: want {want:?}, have {have:?}"),
                );
            }
        }

        if let Some(want) = testcase.stdout.clone() {
            // TODO: fail testcase if stdout is not valid utf8
            let stdout = stdout.lock().unwrap();
            let have = std::str::from_utf8(&stdout)?.to_string();

            if want != have {
                self.add_error(
                    &testcase,
                    format!("stdout mismatch: want {want:?}, have {have:?}"),
                );
            }
        }

        Ok(())
    }

    fn add_error(&mut self, testcase: &TestCase, message: String) {
        self.errors.push(AssertionError {
            filename: self.filename,
            testname: testcase.name.clone(),
            message,
        });
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    Ok,
    Error,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ok => "ok",
                Self::Error => "error",
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct TestCase {
    name: String,
    src: String,
    status: Status,
    /// The length of the vector will be used to check the position of the stack pointer, so we
    /// need to be able to distinguish between stack not provided and empty stack
    stack: Option<Vec<i32>>,
    stdout: Option<String>,
}

pub fn parse_test_file(file: &str) -> Result<Vec<TestCase>> {
    let mut contents = String::new();
    File::open(file)?.read_to_string(&mut contents)?;

    let mut testcases = Vec::new();

    let mut lines = contents.lines().peekable();

    while {
        skip_empty_lines(&mut lines);

        let mut testcase = TestCase::default();

        testcase.name = expect_name(&mut lines)?;
        expect_separator(&mut lines)?;
        testcase.src = read_until_separator(&mut lines);
        expect_separator(&mut lines)?;
        testcase.status = expect_status(&mut lines)?;
        testcase.stack = check_stack(&mut lines)?;
        testcase.stdout = check_stdout(&mut lines)?;

        testcases.push(testcase);

        lines.peek().is_some()
    } {}

    Ok(testcases)
}

fn expect_name(lines: &mut Peekable<Lines<'_>>) -> Result<String> {
    // TODO: use a set to ensure name is unique
    let name = expect_line(lines)?;
    Ok(name.into())
}

fn expect_separator(lines: &mut Peekable<Lines<'_>>) -> Result<()> {
    if expect_line(lines)? != SEPARATOR {
        Err(format!("expected separator"))?
    }

    Ok(())
}

fn expect_status(lines: &mut Peekable<Lines<'_>>) -> Result<Status> {
    let status = match expect_line(lines)? {
        "ok" => Status::Ok,
        "error" => Status::Error,
        status => Err(format!("invalid status: {status}"))?,
    };

    Ok(status)
}

fn read_until_separator(lines: &mut Peekable<Lines<'_>>) -> String {
    let mut s = String::new();
    while let Some(line) = lines.peek() {
        if line.trim() == SEPARATOR {
            break;
        }

        s.extend(line.chars());
        s.push('\n'); // lines() strips the \n which could mess up the program
        lines.next();
    }

    s
}

fn check_stack(lines: &mut Peekable<Lines<'_>>) -> Result<Option<Vec<i32>>> {
    if !check_line(lines)
        .map(|s| s.starts_with("stack"))
        .unwrap_or_default()
    {
        return Ok(None);
    }

    let line = expect_line(lines)?;
    let (_, stack) = line.split_at("stack".len());

    let mut values = Vec::new();

    let mut chars = stack.chars().peekable();
    expect_char(&mut chars, '[')?;
    loop {
        skip_whitespace(&mut chars);

        let s = take_while(&mut chars, |c| ['-', '+'].contains(&c) || c.is_numeric());
        if s.is_empty() {
            break;
        }

        values.push(s.parse::<i32>()?);

        if !check_char(&mut chars, ',') {
            break;
        }
    }
    expect_char(&mut chars, ']')?;

    Ok(Some(values))
}

fn check_stdout(lines: &mut Peekable<Lines<'_>>) -> Result<Option<String>> {
    if !check_line(lines)
        .map(|s| s.starts_with("stdout"))
        .unwrap_or_default()
    {
        return Ok(None);
    }
    expect_line(lines)?;

    let stdout = read_until_separator(lines);
    expect_separator(lines)?;

    Ok(Some(stdout))
}

fn expect_line<'a>(lines: &mut Peekable<Lines<'a>>) -> Result<&'a str> {
    lines
        .next()
        .map(str::trim)
        .ok_or(format!("unexpected eof").into())
}

fn check_line<'a>(lines: &mut Peekable<Lines<'a>>) -> Option<&'a str> {
    lines.peek().map(|s| s.trim())
}

fn expect_char(chars: &mut Peekable<Chars<'_>>, want: char) -> Result<()> {
    skip_whitespace(chars);

    let have = chars.next().ok_or(format!("unexpected eof"))?;
    if want != have {
        Err(format!("want {want}, have {have}"))?
    }

    Ok(())
}

// Unline check_line, check_char will advance the iterator
fn check_char(chars: &mut Peekable<Chars<'_>>, want: char) -> bool {
    skip_whitespace(chars);

    let Some(have) = chars.peek() else {
        return false;
    };

    if want != *have {
        return false;
    }

    chars.next();
    true
}

fn take_while(chars: &mut Peekable<Chars<'_>>, predicate: impl Fn(char) -> bool) -> String {
    let mut s = String::new();
    while let Some(c) = chars.peek() {
        if !predicate(*c) {
            break;
        }

        s.push(*c);
        chars.next();
    }

    s
}

fn skip_whitespace(chars: &mut Peekable<Chars<'_>>) {
    while let Some(c) = chars.peek() {
        if !c.is_whitespace() {
            break;
        }
        chars.next();
    }
}

fn skip_empty_lines(lines: &mut Peekable<Lines<'_>>) {
    while let Some(l) = check_line(lines) {
        if l.is_empty() || l.starts_with("#") {
            lines.next();
            continue;
        }

        break;
    }
}
