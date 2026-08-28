use futures::future::BoxFuture;
use mullvad_management_interface::MullvadProxyClient;
use test_rpc::{ServiceClient, meta::Os};

use crate::mullvad_daemon::RpcClientProvider;

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub func: TestWrapperFunction,
    /// Priority order of the tests, unless specific tests are given as the `TEST_FILTERS` argument
    pub priority: Option<i32>,
    /// A list of location that will be used for by the test
    pub location: Option<Vec<String>>,
    /// If the current test should be skipped.
    pub skip: bool,
}

pub type TestWrapperFunction = fn(
    TestContext,
    ServiceClient,
    Option<MullvadProxyClient>,
) -> BoxFuture<'static, anyhow::Result<()>>;

#[derive(Clone)]
pub struct TestContext {
    pub rpc_provider: RpcClientProvider,
}

impl TestContext {
    pub fn new(rpc_provider: RpcClientProvider) -> Self {
        Self { rpc_provider }
    }
}

#[derive(Clone)]
/// An abbreviated version of [`TestMetadata`]
pub struct TestDescription {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub priority: Option<i32>,
}
