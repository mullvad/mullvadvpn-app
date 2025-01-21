use super::TestWrapperFunction;
use test_rpc::meta::Os;

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub func: TestWrapperFunction,
    pub priority: Option<i32>,
    // TODO: Document
    pub location: Option<Vec<String>>,
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);
