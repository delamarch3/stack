mod testcase;

use crate::testcase::{parse_test_file, TestRunner};

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    const PATH: &str = "tests/files";
    let include_paths = vec![];

    let mut errors = Vec::new();

    for testfile in ["arith.test", "control_flow.test"] {
        let testcases = parse_test_file(&format!("{PATH}/{testfile}"))?;
        let runner = TestRunner::new(testfile, include_paths.clone());
        errors.extend(runner.run(testcases)?);
    }

    if !errors.is_empty() {
        errors.iter().for_each(|e| eprintln!("{e}"));
        panic!("tests failed");
    }

    Ok(())
}
