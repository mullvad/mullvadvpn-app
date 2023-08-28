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
                    .map(|method| mullvad_types::api_access_method::AccessMethod {
                        name: method.name.clone(),
                    })
                    .collect(),
            }
        }
    }

    impl From<proto::ApiAccessMethodAdd> for api_access_method::AccessMethod {
        fn from(value: proto::ApiAccessMethodAdd) -> Self {
            Self { name: value.name }
        }
    }
}

/// Implements conversions for the 'main' AccessMethod data type.
mod data {
    use crate::types::proto;
    use mullvad_types::api_access_method::AccessMethod;

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
            AccessMethod::from(&value)
        }
    }

    impl From<AccessMethod> for proto::ApiAccessMethod {
        fn from(value: AccessMethod) -> Self {
            Self {
                name: value.name.clone(),
            }
        }
    }

    impl From<&proto::ApiAccessMethod> for AccessMethod {
        fn from(value: &proto::ApiAccessMethod) -> Self {
            Self {
                name: value.name.clone(),
            }
        }
    }

    impl From<Vec<AccessMethod>> for proto::ApiAccessMethods {
        fn from(value: Vec<AccessMethod>) -> proto::ApiAccessMethods {
            proto::ApiAccessMethods {
                api_access_methods: value.iter().map(|method| method.clone().into()).collect(),
            }
        }
    }
}
