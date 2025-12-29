use std::{
    fs::File,
    io::Read,
    iter::Peekable,
    str::{Chars, Lines},
};

const SEPARATOR: &str = "----";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default)]
pub struct TestCase {
    name: String,
    src: String,
    ok: bool,
    /// The length of the vector will be used to check the position of the stack pointer, so we
    /// need to be able to distinguish between stack not provided and empty stack
    stack: Option<Vec<u64>>,
    stdout: Option<String>,
}

pub fn parse_file(file: &str) -> Result<Vec<TestCase>> {
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
        testcase.ok = expect_status(&mut lines)?;
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

fn expect_status(lines: &mut Peekable<Lines<'_>>) -> Result<bool> {
    let ok = match expect_line(lines)? {
        "ok" => true,
        "error" => false,
        status => Err(format!("invalid status: {status}"))?,
    };

    Ok(ok)
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

fn check_stack(lines: &mut Peekable<Lines<'_>>) -> Result<Option<Vec<u64>>> {
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

        let s = take_while(&mut chars, char::is_numeric);
        if s.is_empty() {
            break;
        }

        values.push(s.parse::<u64>()?);

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
