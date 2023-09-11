/// Implements conversions for the auxilliary proto AccessMethod type to the internal AccessMethod data type.
mod settings {
    use crate::types::proto;
    use mullvad_types::api_access_method;

    impl From<&api_access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: &api_access_method::Settings) -> Self {
            Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(|method| method.clone().into())
                    .collect(),
            }
        }
    }

    impl From<api_access_method::Settings> for proto::ApiAccessMethodSettings {
        fn from(settings: api_access_method::Settings) -> Self {
            proto::ApiAccessMethodSettings::from(&settings)
        }
    }

    impl From<proto::ApiAccessMethodSettings> for api_access_method::Settings {
        fn from(settings: proto::ApiAccessMethodSettings) -> Self {
            Self {
                api_access_methods: settings
                    .api_access_methods
                    .iter()
                    .map(api_access_method::AccessMethod::from)
                    .collect(),
            }
        }
    }

    impl From<api_access_method::ApiAccessMethodReplace> for proto::ApiAccessMethodReplace {
        fn from(value: api_access_method::ApiAccessMethodReplace) -> Self {
            proto::ApiAccessMethodReplace {
                index: value.index as u32,
                access_method: Some(value.access_method.into()),
            }
        }
    }

    impl From<proto::ApiAccessMethodReplace> for api_access_method::ApiAccessMethodReplace {
        // TODO: Implement `TryFrom` instead, and skip the `unwrap`.
        fn from(value: proto::ApiAccessMethodReplace) -> Self {
            api_access_method::ApiAccessMethodReplace {
                index: value.index as usize,
                access_method: value.access_method.unwrap().into(),
            }
        }
    }
}

/// Implements conversions for the 'main' AccessMethod data type.
mod data {
    use crate::types::proto::{self, api_access_method::socks5::Socks5type};
    use mullvad_types::api_access_method::{
        AccessMethod, Shadowsocks, Socks5, Socks5Local, Socks5Remote,
    };

    impl From<proto::ApiAccessMethods> for Vec<AccessMethod> {
        fn from(api_access_methods: proto::ApiAccessMethods) -> Self {
            api_access_methods
                .api_access_methods
                .iter()
                .map(AccessMethod::from)
                .collect()
        }
    }

    impl From<proto::ApiAccessMethod> for AccessMethod {
        fn from(value: proto::ApiAccessMethod) -> Self {
            // TODO: How to not unwrap?
            match value.access_method.unwrap() {
                proto::api_access_method::AccessMethod::Socks5(socks) => {
                    match socks.socks5type.unwrap() {
                        Socks5type::Local(local) => {
                            let local_proxy = Socks5Local::from_args(
                                local.ip,
                                local.port as u16,
                                local.local_port as u16,
                            )
                            .unwrap(); // This is dangerous territory ..
                            AccessMethod::Socks5(Socks5::Local(local_proxy))
                        }

                        Socks5type::Remote(remote) => {
                            let remote_proxy =
                                Socks5Remote::from_args(remote.ip, remote.port as u16).unwrap(); // This is dangerous territory ..
                            AccessMethod::Socks5(Socks5::Remote(remote_proxy))
                        }
                    }
                }
                proto::api_access_method::AccessMethod::Shadowsocks(ss) => {
                    let shadow_sock =
                        Shadowsocks::from_args(ss.ip, ss.port as u16, ss.cipher, ss.password)
                            .unwrap();
                    AccessMethod::Shadowsocks(shadow_sock)
                }
            }
        }
    }

    impl From<AccessMethod> for proto::ApiAccessMethod {
        fn from(value: AccessMethod) -> Self {
            match value {
                AccessMethod::Shadowsocks(ss) => proto::api_access_method::Shadowsocks {
                    ip: ss.peer.ip().to_string(),
                    port: ss.peer.port() as u32,
                    password: ss.password,
                    cipher: ss.cipher,
                }
                .into(),

                AccessMethod::Socks5(Socks5::Local(Socks5Local { peer, port })) => {
                    proto::api_access_method::Socks5Local {
                        ip: peer.ip().to_string(),
                        port: peer.port() as u32,
                        local_port: port as u32,
                    }
                    .into()
                }
                AccessMethod::Socks5(Socks5::Remote(Socks5Remote { peer })) => {
                    proto::api_access_method::Socks5Remote {
                        ip: peer.ip().to_string(),
                        port: peer.port() as u32,
                    }
                    .into()
                }
            }
        }
    }

    impl From<&proto::ApiAccessMethod> for AccessMethod {
        fn from(value: &proto::ApiAccessMethod) -> Self {
            AccessMethod::from(value.clone())
        }
    }

    impl From<Vec<AccessMethod>> for proto::ApiAccessMethods {
        fn from(value: Vec<AccessMethod>) -> proto::ApiAccessMethods {
            proto::ApiAccessMethods {
                api_access_methods: value.iter().map(|method| method.clone().into()).collect(),
            }
        }
    }

    impl From<proto::api_access_method::Shadowsocks> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Shadowsocks) -> Self {
            proto::api_access_method::AccessMethod::Shadowsocks(value).into()
        }
    }

    impl From<proto::api_access_method::Socks5> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5) -> Self {
            proto::api_access_method::AccessMethod::Socks5(value).into()
        }
    }

    impl From<proto::api_access_method::socks5::Socks5type> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::socks5::Socks5type) -> Self {
            proto::api_access_method::AccessMethod::Socks5(proto::api_access_method::Socks5 {
                socks5type: Some(value),
            })
            .into()
        }
    }

    impl From<proto::api_access_method::Socks5Local> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5Local) -> Self {
            proto::api_access_method::socks5::Socks5type::Local(value).into()
        }
    }

    impl From<proto::api_access_method::Socks5Remote> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::Socks5Remote) -> Self {
            proto::api_access_method::socks5::Socks5type::Remote(value).into()
        }
    }

    impl From<proto::api_access_method::AccessMethod> for proto::ApiAccessMethod {
        fn from(value: proto::api_access_method::AccessMethod) -> Self {
            proto::ApiAccessMethod {
                access_method: Some(value),
            }
        }
    }
}
