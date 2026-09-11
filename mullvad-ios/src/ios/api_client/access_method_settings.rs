use crate::api_client::helpers::{ShadowsocksWrapper, Socks5RemoteWrapper};
use mullvad_types::access_method::{
    AccessMethod, AccessMethodSetting,
    BuiltInAccessMethod::{Bridge, Direct, EncryptedDnsProxy},
    Id, Settings,
};
use std::sync::Arc;
use talpid_types::net::proxy;

/// Converts parameters into a `Box<AccessMethodSetting>` raw representation that
/// can be passed across the FFI boundary
#[uniffi::export]
pub fn convert_builtin_access_method_setting(
    unique_identifier: String,
    name: String,
    is_enabled: bool,
    method_kind: SwiftAccessMethodKind,
) -> Option<Arc<AccessMethodSettingWrapper>> {
    convert_builtin_access_method_setting_inner(unique_identifier, name, is_enabled, method_kind)
        .map(|access_method| {
            Arc::new(AccessMethodSettingWrapper {
                inner: access_method,
            })
        })
}

/// Converts parameters into an `AccessMethodSetting`
///
/// This function copies the strings from the conversion of the variables
/// `unique_identifier`, `name`, and takes ownership of `proxy_configuration`
fn convert_builtin_access_method_setting_inner(
    unique_identifier: String,
    name: String,
    enabled: bool,
    method_kind: SwiftAccessMethodKind,
) -> Option<AccessMethodSetting> {
    let id = Id::from_string(unique_identifier)?;
    match method_kind {
        SwiftAccessMethodKind::KindDirect => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(Direct),
        )),
        SwiftAccessMethodKind::KindBridge => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(Bridge),
        )),

        SwiftAccessMethodKind::KindEncryptedDnsProxy => Some(AccessMethodSetting::with_id(
            id,
            name,
            enabled,
            AccessMethod::BuiltIn(EncryptedDnsProxy),
        )),

        SwiftAccessMethodKind::KindShadowsocks(configuration) => Some({
            AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                AccessMethod::Custom(proxy::CustomProxy::Shadowsocks(configuration.0.clone())),
            )
        }),
        SwiftAccessMethodKind::KindSocks5Local(configuration) => Some({
            AccessMethodSetting::with_id(
                id,
                name,
                enabled,
                AccessMethod::Custom(proxy::CustomProxy::Socks5Remote(configuration.0.clone())),
            )
        }),
    }
}

#[uniffi::export(with_foreign)]
pub trait ShadowsocksBridgeProvider: Sync + Send {
    fn bridge(&self) -> Option<Arc<ShadowsocksWrapper>>;
}

/// Used by Swift to instruct which access method kind it is trying to convert

#[derive(uniffi::Enum)]
pub enum SwiftAccessMethodKind {
    KindDirect,
    KindBridge,
    KindEncryptedDnsProxy,
    KindShadowsocks(Arc<ShadowsocksWrapper>),
    KindSocks5Local(Arc<Socks5RemoteWrapper>),
}

#[derive(uniffi::Object)]
pub struct AccessMethodSettingWrapper {
    pub inner: AccessMethodSetting,
}

/// Creates a wrapper around a `Settings` object that can be safely sent across the FFI boundary.
#[uniffi::export]
pub fn init_access_method_settings_wrapper(
    direct: Arc<AccessMethodSettingWrapper>,
    bridges: Arc<AccessMethodSettingWrapper>,
    encrypted_dns: Arc<AccessMethodSettingWrapper>,
    custom: Vec<Arc<AccessMethodSettingWrapper>>,
) -> SwiftAccessMethodSettingsContext {
    let settings = Settings::new(
        direct.inner.clone(),
        bridges.inner.clone(),
        encrypted_dns.inner.clone(),
        custom.into_iter().map(|a| a.inner.clone()).collect(),
    );
    SwiftAccessMethodSettingsContext { settings }
}

#[derive(Debug, uniffi::Object)]
pub struct SwiftAccessMethodSettingsContext {
    pub settings: Settings,
}

impl SwiftAccessMethodSettingsContext {
    pub fn convert_access_method(&self) -> Option<Settings> {
        Some(self.settings.clone())
    }
}
