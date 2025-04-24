use chrono::{DateTime, Utc};
use http::StatusCode;
use hyper::body::Incoming;
use mullvad_types::{
    account::AccountNumber,
    device::{Device, DeviceId, DeviceName},
};
use std::future::Future;
use talpid_types::net::wireguard;

use crate::rest;

use super::ACCOUNTS_URL_PREFIX;

#[derive(Clone)]
pub struct DevicesProxy {
    handle: rest::MullvadRestHandle,
}

#[derive(serde::Deserialize)]
struct DeviceResponse {
    id: DeviceId,
    name: DeviceName,
    pubkey: wireguard::PublicKey,
    ipv4_address: ipnetwork::Ipv4Network,
    ipv6_address: ipnetwork::Ipv6Network,
    hijack_dns: bool,
    created: DateTime<Utc>,
}

impl DevicesProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn create(
        &self,
        account: AccountNumber,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<
        Output = Result<(Device, mullvad_types::wireguard::AssociatedAddresses), rest::Error>,
    > + use<> {
        let request = self.create_response(account, pubkey);

        async move {
            let DeviceResponse {
                id,
                name,
                pubkey,
                ipv4_address,
                ipv6_address,
                hijack_dns,
                created,
                ..
            } = request.await?.deserialize().await?;

            Ok((
                Device {
                    id,
                    name,
                    pubkey,
                    hijack_dns,
                    created,
                },
                mullvad_types::wireguard::AssociatedAddresses {
                    ipv4_address,
                    ipv6_address,
                },
            ))
        }
    }

    pub fn get(
        &self,
        account: AccountNumber,
        id: DeviceId,
    ) -> impl Future<Output = Result<Device, rest::Error>> + use<> {
        let request = self.get_response(account, id);
        async move {
            let data = request.await?.deserialize().await?;
            Ok(data)
        }
    }

    pub fn list(
        &self,
        account: AccountNumber,
    ) -> impl Future<Output = Result<Vec<Device>, rest::Error>> + use<> {
        let request = self.list_response(account);
        async move {
            let data = request.await?.deserialize().await?;
            Ok(data)
        }
    }

    pub fn remove(
        &self,
        account: AccountNumber,
        id: DeviceId,
    ) -> impl Future<Output = Result<(), rest::Error>> + use<> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        async move {
            let request = factory
                .delete(&format!("{ACCOUNTS_URL_PREFIX}/devices/{id}"))?
                .expected_status(&[StatusCode::NO_CONTENT])
                .account(account)?;
            service.request(request).await?;
            Ok(())
        }
    }

    pub fn replace_wg_key(
        &self,
        account: AccountNumber,
        id: DeviceId,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<Output = Result<mullvad_types::wireguard::AssociatedAddresses, rest::Error>> + use<>
    {
        let request = self.replace_wg_key_response(account, id, pubkey);
        async move {
            let DeviceResponse {
                ipv4_address,
                ipv6_address,
                ..
            } = request.await?.deserialize().await?;
            Ok(mullvad_types::wireguard::AssociatedAddresses {
                ipv4_address,
                ipv6_address,
            })
        }
    }

    pub fn get_response(
        &self,
        account: AccountNumber,
        id: DeviceId,
    ) -> impl Future<Output = Result<rest::Response<Incoming>, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .get(&format!("{ACCOUNTS_URL_PREFIX}/devices/{id}"))?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            service.request(request).await
        }
    }

    pub fn list_response(
        &self,
        account: AccountNumber,
    ) -> impl Future<Output = Result<rest::Response<Incoming>, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .get(&format!("{ACCOUNTS_URL_PREFIX}/devices"))?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            service.request(request).await
        }
    }

    pub fn replace_wg_key_response(
        &self,
        account: AccountNumber,
        id: DeviceId,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<Output = Result<rest::Response<Incoming>, rest::Error>> {
        #[derive(serde::Serialize)]
        struct RotateDevicePubkey {
            pubkey: wireguard::PublicKey,
        }
        let req_body = RotateDevicePubkey { pubkey };

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .put_json(
                    &format!("{ACCOUNTS_URL_PREFIX}/devices/{id}/pubkey"),
                    &req_body,
                )?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            service.request(request).await
        }
    }

    pub fn create_response(
        &self,
        account: AccountNumber,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<Output = Result<rest::Response<Incoming>, rest::Error>> {
        #[derive(serde::Serialize)]
        struct DeviceSubmission {
            pubkey: wireguard::PublicKey,
            hijack_dns: bool,
        }

        let submission = DeviceSubmission {
            pubkey,
            hijack_dns: false,
        };

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .post_json(&format!("{ACCOUNTS_URL_PREFIX}/devices"), &submission)?
                .account(account)?
                .expected_status(&[StatusCode::CREATED]);
            service.request(request).await
        }
    }
}
