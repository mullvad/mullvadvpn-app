extern crate jsonrpc_core;
extern crate serde_json;

use serde;

static mut MOCK_IS_CONNECTED: bool = false;
type Result<T> = ::std::result::Result<T, String>;

pub fn build_router() -> jsonrpc_core::IoHandler {
    let mut router = jsonrpc_core::IoHandler::default();

    add_route(&mut router, "login", mock_login);
    add_route(&mut router, "logout", mock_logout);
    add_route(&mut router, "connect", mock_connect);
    add_route(&mut router, "disconnect", mock_disconnect);
    add_route(&mut router, "get_connection", mock_get_connection_info);
    add_route(&mut router, "get_location", mock_get_location);

    router
}

fn add_route<T, U, F>(router: &mut jsonrpc_core::IoHandler, method: &str, handler: F)
    where T: serde::Deserialize + 'static,
          U: serde::Serialize + 'static,
          F: Fn(&T) -> Result<U> + Send + Sync + 'static
{
    let c = move |params: jsonrpc_core::params::Params| {
        println!("Got rpc request with params {:?}", params);
        let parsed_params: T = params.parse()?;

        let response: U = handler(&parsed_params)
            .map_err(
                |e| {
                    error!("Failed responding to RPC request: {}", e);
                    jsonrpc_core::Error::internal_error()
                },
            )?;

        serde_json::to_value(response).map_err(
            |e| {
                error!("Unable to serialize response to RPC request: {}", e);
                jsonrpc_core::Error::internal_error()
            },
        )
    };
    router.add_method(method, c);
}

#[derive(Deserialize)]
struct LoginRequest {
    #[serde(rename = "accountNumber")]
    account_number: String,
}
fn mock_login(request: &LoginRequest) -> Result<::std::collections::HashMap<String, String>> {
    let ref account_number = request.account_number;

    let mut reply = ::std::collections::HashMap::new();

    if account_number.starts_with("1111") {
        // accounts starting with 1111 expire in one month
        reply.insert(
            "paidUntil".to_owned(),
            "2018-12-31T16:00:00.000Z".to_owned(),
        );
        // res.paidUntil = moment().startOf('day').add(15, 'days').toISOString();
    } else if account_number.starts_with("2222") {
        // expired in 2013
        reply.insert(
            "paidUntil".to_owned(),
            "2012-12-31T16:00:00.000Z".to_owned(),
        );
    } else if account_number.starts_with("3333") {
        // expire in 2038
        reply.insert(
            "paidUntil".to_owned(),
            "2037-12-31T16:00:00.000Z".to_owned(),
        );
    } else {
        bail!("you are not welcome {}!", account_number)
    }

    Ok(reply)
}

fn mock_logout(_: &()) -> Result<()> {
    Ok(())
}

#[derive(Deserialize)]
struct ConnectRequest {
    address: String,
}
fn mock_connect(request: &ConnectRequest) -> Result<()> {
    let ref server_address = request.address;
    if server_address.starts_with("se") {
        bail!("{} is unreachable", server_address)
    }

    unsafe { MOCK_IS_CONNECTED = true };
    Ok(())
}

fn mock_disconnect(_: &()) -> Result<()> {
    unsafe { MOCK_IS_CONNECTED = false };
    Ok(())
}

#[derive(Serialize)]
struct ConnectionInfo {
    ip: String,
}
fn mock_get_connection_info(_: &()) -> Result<ConnectionInfo> {
    let ip = if unsafe { MOCK_IS_CONNECTED } {
            "1.2.3.4"
        } else {
            "192.168.1.2"
        }
        .to_owned();

    Ok(ConnectionInfo { ip: ip })
}

#[derive(Serialize)]
struct Location {
    latlong: [u32; 2],
    country: String,
    city: String,
}
fn mock_get_location(_: &()) -> Result<Location> {
    let response = if unsafe { MOCK_IS_CONNECTED } {
        Location {
            latlong: [1, 2],
            country: "narnia".to_owned(),
            city: "Le city".to_owned(),
        }
    } else {
        Location {
            latlong: [60, 61],
            country: "sweden".to_owned(),
            city: "bollebygd".to_owned(),
        }
    };

    Ok(response)
}
