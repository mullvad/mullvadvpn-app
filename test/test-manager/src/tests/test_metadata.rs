use super::TestWrapperFunction;
use test_rpc::meta::Os;

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub func: TestWrapperFunction,
    /// Priority order of the tests, unless specific tests are given as the `TEST_FILTERS` argument
    pub priority: Option<i32>,
    /// A list of location that will be used for by the test
    pub location: Option<Vec<String>>,
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);
