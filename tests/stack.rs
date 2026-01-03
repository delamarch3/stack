mod testcase;

use std::{fs::read_dir, io, path::PathBuf};

use crate::testcase::{parse_test_file, TestRunner};

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    const TESTS: &str = "tests/files/tests";
    let include_paths = vec![PathBuf::from("tests/files/include")];

    let mut errors = Vec::new();

    let testfiles = read_dir(TESTS)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    for testfile in testfiles {
        let testcases = parse_test_file(&testfile)?;
        let runner = TestRunner::new(
            testfile.to_str().map(String::from).unwrap(),
            include_paths.clone(),
        );
        errors.extend(runner.run(testcases)?);
    }

    if !errors.is_empty() {
        errors.iter().for_each(|e| eprintln!("{e}"));
        panic!("{len} assertions failed", len = errors.len());
    }

    Ok(())
}
