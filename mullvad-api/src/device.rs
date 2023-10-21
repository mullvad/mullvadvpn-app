use chrono::{DateTime, Utc};
use http::StatusCode;
use mullvad_types::{
    account::AccountToken,
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
        account: AccountToken,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<Output = Result<(Device, mullvad_types::wireguard::AssociatedAddresses), rest::Error>>
    {
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
            let response = service.request(request).await;

            let response: DeviceResponse = rest::deserialize_body(response?).await?;
            let DeviceResponse {
                id,
                name,
                pubkey,
                ipv4_address,
                ipv6_address,
                hijack_dns,
                created,
                ..
            } = response;

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
        account: AccountToken,
        id: DeviceId,
    ) -> impl Future<Output = Result<Device, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        async move {
            let request = factory
                .get(&format!("{ACCOUNTS_URL_PREFIX}/devices/{id}"))?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            let response = service.request(request).await?;
            let device = rest::deserialize_body(response).await?;
            Ok(device)
        }
    }

    pub fn list(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<Vec<Device>, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        async move {
            let request = factory
                .get(&format!("{ACCOUNTS_URL_PREFIX}/device"))?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            let response = service.request(request).await?;
            let devices = rest::deserialize_body(response).await?;
            Ok(devices)
        }
    }

    pub fn remove(
        &self,
        account: AccountToken,
        id: DeviceId,
    ) -> impl Future<Output = Result<(), rest::Error>> {
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
        account: AccountToken,
        id: DeviceId,
        pubkey: wireguard::PublicKey,
    ) -> impl Future<Output = Result<mullvad_types::wireguard::AssociatedAddresses, rest::Error>>
    {
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
            let response = service.request(request).await?;

            let updated_device: DeviceResponse = rest::deserialize_body(response).await?;
            let DeviceResponse {
                ipv4_address,
                ipv6_address,
                ..
            } = updated_device;
            Ok(mullvad_types::wireguard::AssociatedAddresses {
                ipv4_address,
                ipv6_address,
            })
        }
    }
}
