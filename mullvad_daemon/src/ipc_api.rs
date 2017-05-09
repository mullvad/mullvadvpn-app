use jsonrpc_core::Error;
use jsonrpc_core::futures::BoxFuture;
use jsonrpc_macros::pubsub;
use jsonrpc_pubsub::SubscriptionId;

use std::collections::HashMap;
use std::net::IpAddr;

pub type AccountToken = String;
pub type CountryCode = String;

build_rpc_trait! {
    pub trait IpcApi {
        type Metadata;

        /// Fetches and returns metadata about an account. Returns an error on non-existing
        /// accounts.
        #[rpc(name = "get_account_data")]
        fn get_account_data(&self, AccountToken) -> Result<AccountData, Error>;

        /// Returns available countries.
        #[rpc(name = "get_countries")]
        fn get_countries(&self) -> Result<HashMap<CountryCode, String>, Error>;

        /// Set which account to connect with
        #[rpc(name = "set_account")]
        fn set_account(&self, AccountToken) -> Result<(), Error>;

        /// Set which country to connect to
        #[rpc(name = "set_country")]
        fn set_country(&self, CountryCode) -> Result<(), Error>;

        /// Set if the backend should automatically establish a tunnel on start or not.
        #[rpc(name = "set_autoconnect")]
        fn set_autoconnect(&self, bool) -> Result<(), Error>;

        /// Try to connect if disconnected, or do nothing if already connecting/connected.
        #[rpc(name = "connect")]
        fn connect(&self) -> Result<(), Error>;

        /// Disconnect the VPN tunnel if it is connecting/connected. Does nothing if already
        /// disconnected.
        #[rpc(name = "disconnect")]
        fn disconnect(&self) -> Result<(), Error>;

        /// Returns the current security state of the Mullvad client. Changes to this state will
        /// be announced to subscribers of `event`.
        #[rpc(name = "get_state")]
        fn get_state(&self) -> Result<SecurityState, Error>;

        /// Returns the current public IP of this computer.
        #[rpc(name = "get_ip")]
        fn get_ip(&self) -> Result<IpAddr, Error>;

        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(name = "get_location")]
        fn get_location(&self) -> Result<Location, Error>;

        #[pubsub(name = "event")] {
            /// Subscribes to the `event` notifications.
            #[rpc(name = "event_subscribe")]
            fn subscribe(&self, Self::Metadata, pubsub::Subscriber<String>);

            /// Unsubscribes from the `event` notifications.
            #[rpc(name = "event_unsubscribe")]
            fn unsubscribe(&self, SubscriptionId) -> BoxFuture<bool, Error>;
        }
    }
}

#[derive(Serialize)]
pub struct AccountData {
    pub paid_until: String,
}

#[derive(Serialize)]
pub struct Location {
    pub latlong: [f64; 2],
    pub country: String,
    pub city: String,
}

#[derive(Serialize)]
pub enum SecurityState {
    Unsecured,
    Securing,
    Secured,
    Unsecuring,
}
