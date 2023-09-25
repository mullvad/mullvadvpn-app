/// A short-lived datastructure used in the `ApiAccessMethodUpdate` RPC call.
use mullvad_types::access_method::{ApiAccessMethod, ApiAccessMethodId};
/// Argument to gRPC call `UpdateApiAccessMethod`.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiAccessMethodUpdate {
    pub id: ApiAccessMethodId,
    pub access_method: ApiAccessMethod,
}
