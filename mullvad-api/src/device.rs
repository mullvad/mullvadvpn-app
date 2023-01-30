use chrono::{DateTime, Utc};
use http::{Method, StatusCode};
use mullvad_types::{
    account::AccountToken,
    device::{Device, DeviceId, DeviceName, DevicePort},
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
    ports: Vec<DevicePort>,
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
        let access_proxy = self.handle.token_store.clone();

        async move {
            let response = rest::send_json_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/devices"),
                Method::POST,
                &submission,
                Some((access_proxy, account)),
                &[StatusCode::CREATED],
            )
            .await;

            let response: DeviceResponse = rest::deserialize_body(response?).await?;
            let DeviceResponse {
                id,
                name,
                pubkey,
                ipv4_address,
                ipv6_address,
                ports,
                hijack_dns,
                created,
                ..
            } = response;

            Ok((
                Device {
                    id,
                    name,
                    pubkey,
                    ports,
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
        let access_proxy = self.handle.token_store.clone();
        async move {
            let response = rest::send_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/devices/{id}"),
                Method::GET,
                Some((access_proxy, account)),
                &[StatusCode::OK],
            )
            .await;
            rest::deserialize_body(response?).await
        }
    }

    pub fn list(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<Vec<Device>, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let access_proxy = self.handle.token_store.clone();
        async move {
            let response = rest::send_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/devices"),
                Method::GET,
                Some((access_proxy, account)),
                &[StatusCode::OK],
            )
            .await;
            rest::deserialize_body(response?).await
        }
    }

    pub fn remove(
        &self,
        account: AccountToken,
        id: DeviceId,
    ) -> impl Future<Output = Result<(), rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let access_proxy = self.handle.token_store.clone();
        async move {
            let response = rest::send_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/devices/{id}"),
                Method::DELETE,
                Some((access_proxy, account)),
                &[StatusCode::NO_CONTENT],
            )
            .await;

            response?;
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
        let access_proxy = self.handle.token_store.clone();

        async move {
            let response = rest::send_json_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/devices/{id}/pubkey"),
                Method::PUT,
                &req_body,
                Some((access_proxy, account)),
                &[StatusCode::OK],
            )
            .await;

            let updated_device: DeviceResponse = rest::deserialize_body(response?).await?;
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
