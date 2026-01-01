mod testcase;

use crate::testcase::{parse_test_file, TestRunner};

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    const FILENAME: &str = "tests/files/arith.test";

    let testcases = parse_test_file(FILENAME)?;

    let runner = TestRunner::new(FILENAME, vec![]);

    let errors = runner.run(testcases)?;

    if !errors.is_empty() {
        errors.iter().for_each(|e| eprintln!("{e}"));
        panic!("tests failed");
    }

    Ok(())
}
