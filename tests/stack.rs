mod testcase;

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    let testcases = testcase::parse_file("tests/files/arith.test")?;

    for testcase in testcases {
        dbg!(&testcase);
    }

    panic!();

    Ok(())
}
