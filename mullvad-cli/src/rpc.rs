use serde::{Deserialize, Serialize};

use super::Result;

use mullvad_ipc_client::DaemonRpcClient;

pub fn call<T, O>(method: &str, args: &T) -> Result<O>
where
    T: Serialize,
    O: for<'de> Deserialize<'de>,
{
    let rpc = DaemonRpcClient::new()?;
    Ok(rpc.call(method, args)?)
}
