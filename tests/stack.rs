mod testcase;

use crate::testcase::{parse_test_file, TestRunner};

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    const FILENAME: &str = "tests/files/arith.test";

    let testcases = parse_test_file(FILENAME)?;

    let runner = TestRunner::new(FILENAME, testcases, vec![]);

    let errors = runner.run()?;

    if !errors.is_empty() {
        dbg!(errors);
        panic!("tests failed");
    }

    Ok(())
}
