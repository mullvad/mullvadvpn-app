use mullvad_management_interface::MullvadProxyClient;
use test_macro::test_function;
use test_rpc::ServiceClient;

use super::TestContext;

#[test_function(target_os = "linux")]
pub async fn test_cve_2019_14899_mitigation(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    todo!()
}
