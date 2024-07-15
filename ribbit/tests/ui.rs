#[test]
fn main() {
    let runner = trybuild::TestCases::new();
    runner.compile_fail("tests/ui/*.rs");
    runner.pass("*.rs");
}
