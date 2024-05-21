//! This module provides functionality for updating settings using a JSON string, i.e. applying a
//! patch. It is intended to be relatively safe, preventing editing of "dangerous" settings such as
//! custom DNS.
//!
//! Patching the settings is a three-step procedure:
//! 1. Validating the input. Only a subset of settings is allowed to be edited using this method.
//!    Attempting to edit prohibited or invalid settings results in an error.
//! 2. Merging the changes. When the patch has been accepted, it can be applied to the existing
//!    settings. How they're merged depends on the actual setting. See [MergeStrategy].
//! 3. Deserialize the resulting JSON back to a [Settings] instance, and, if valid, replace the
//!    existing settings.
//!
//! Permitted settings and merge strategies are defined in the [PERMITTED_SUBKEYS] constant.
//!
//! This implementation must be kept in sync with the
//! [spec](../../../docs/settings-patch-format.md).

use super::SettingsPersister;
use mullvad_types::settings::Settings;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Missing expected JSON object
    #[error("Incorrect or missing value: {0}")]
    InvalidOrMissingValue(&'static str),
    /// Unknown or prohibited key
    #[error("Invalid or prohibited key: {0}")]
    UnknownOrProhibitedKey(String),
    /// Failed to parse patch json
    #[error("Failed to parse settings patch")]
    ParsePatch(#[source] serde_json::Error),
    /// Failed to deserialize patched settings
    #[error("Failed to deserialize patched settings")]
    DeserializePatched(#[source] serde_json::Error),
    /// Failed to serialize settings
    #[error("Failed to serialize current settings")]
    SerializeSettings(#[source] serde_json::Error),
    /// Failed to serialize field
    #[error("Failed to serialize value")]
    SerializeValue(#[source] serde_json::Error),
    /// Recursion limit reached
    #[error("Maximum JSON object depth reached")]
    RecursionLimit,
    /// Settings error
    #[error("Settings error")]
    Settings(#[source] super::Error),
}

/// Converts an [Error] to a management interface status
impl From<Error> for mullvad_management_interface::Status {
    fn from(error: Error) -> mullvad_management_interface::Status {
        use mullvad_management_interface::Status;

        match error {
            Error::InvalidOrMissingValue(_)
            | Error::UnknownOrProhibitedKey(_)
            | Error::ParsePatch(_)
            | Error::DeserializePatched(_)
            | Error::RecursionLimit => Status::invalid_argument(error.to_string()),
            Error::Settings(error) => Status::from(error),
            Error::SerializeSettings(error) | Error::SerializeValue(error) => {
                Status::internal(error.to_string())
            }
        }
    }
}

enum MergeStrategy {
    /// Replace or append keys to objects, and replace everything else
    Replace,
    /// Call a function to combine an existing setting (which may be null) with the patch.
    /// The returned value replaces the existing node.
    Custom(fn(&serde_json::Value, &serde_json::Value) -> Result<serde_json::Value, Error>),
}

// TODO: Use Default trait when `const_trait_impl`` is available.
const DEFAULT_MERGE_STRATEGY: MergeStrategy = MergeStrategy::Replace;

struct PermittedKey {
    key_type: PermittedKeyValue,
    merge_strategy: MergeStrategy,
}

impl PermittedKey {
    const fn object(keys: &'static [(&'static str, PermittedKey)]) -> Self {
        Self {
            key_type: PermittedKeyValue::Object(keys),
            merge_strategy: DEFAULT_MERGE_STRATEGY,
        }
    }

    const fn array(key: &'static PermittedKey) -> Self {
        Self {
            key_type: PermittedKeyValue::Array(key),
            merge_strategy: DEFAULT_MERGE_STRATEGY,
        }
    }

    const fn any() -> Self {
        Self {
            key_type: PermittedKeyValue::Any,
            merge_strategy: DEFAULT_MERGE_STRATEGY,
        }
    }

    const fn merge_strategy(mut self, merge_strategy: MergeStrategy) -> Self {
        self.merge_strategy = merge_strategy;
        self
    }
}

enum PermittedKeyValue {
    /// Select subkeys that can be modified at this level
    Object(&'static [(&'static str, PermittedKey)]),
    /// Array that can be modified at this level
    Array(&'static PermittedKey),
    /// Accept any object at this level
    Any,
}

const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[(
    "relay_overrides",
    PermittedKey::array(&PermittedKey::object(&[
        ("hostname", PermittedKey::any()),
        ("ipv4_addr_in", PermittedKey::any()),
        ("ipv6_addr_in", PermittedKey::any()),
    ]))
    .merge_strategy(MergeStrategy::Custom(merge_relay_overrides)),
)]);
/// Prohibit stack overflow via excessive recursion. It might be possible to forgo this when
/// tail-call optimization can be enforced?
const RECURSE_LIMIT: usize = 15;

/// Export a patch containing all currently supported settings.
pub fn export_settings(settings: &Settings) -> Result<String, Error> {
    let patch = export_settings_inner(settings)?;
    serde_json::to_string_pretty(&patch).map_err(Error::SerializeValue)
}

fn export_settings_inner(settings: &Settings) -> Result<serde_json::Value, Error> {
    let mut out = serde_json::Map::new();
    let mut overrides = vec![];

    for relay_override in &settings.relay_overrides {
        let mut relay_override =
            serde_json::to_value(relay_override).map_err(Error::SerializeValue)?;
        if let Some(relay_overrides) = relay_override.as_object_mut() {
            // prune empty override entries
            relay_overrides.retain(|_k, v| !v.is_null());
            let has_overrides = relay_overrides.iter().any(|(key, _)| key != "hostname");
            if !has_overrides {
                continue;
            }
        }
        overrides.push(relay_override);
    }

    if !overrides.is_empty() {
        out.insert(
            "relay_overrides".to_owned(),
            serde_json::Value::Array(overrides),
        );
    }

    Ok(serde_json::Value::Object(out))
}

/// Update the settings with the supplied patch. Only settings specified in `PERMITTED_SUBKEYS` can
/// be updated. All other changes are rejected
pub async fn merge_validate_patch(
    settings: &mut SettingsPersister,
    json_patch: &str,
) -> Result<(), Error> {
    let new_settings = merge_validate_patch_inner(settings, json_patch)?;

    settings
        .update(move |settings| *settings = new_settings)
        .await
        .map_err(Error::Settings)?;

    Ok(())
}

fn merge_validate_patch_inner(settings: &Settings, json_patch: &str) -> Result<Settings, Error> {
    let mut settings_value: serde_json::Value =
        serde_json::to_value(settings).map_err(Error::SerializeSettings)?;
    let patch_value: serde_json::Value =
        serde_json::from_str(json_patch).map_err(Error::ParsePatch)?;

    validate_patch_value(PERMITTED_SUBKEYS, &patch_value, 0)?;
    merge_patch_to_value(PERMITTED_SUBKEYS, &mut settings_value, &patch_value, 0)?;

    let new_settings: Settings =
        serde_json::from_value(settings_value).map_err(Error::DeserializePatched)?;

    Ok(new_settings)
}

/// Replace overrides for existing values in the array if there's a matching hostname. For hostnames
/// that do not exist, just append the overrides.
fn merge_relay_overrides(
    current_settings: &serde_json::Value,
    patch: &serde_json::Value,
) -> Result<serde_json::Value, Error> {
    if current_settings.is_null() {
        return Ok(patch.to_owned());
    }

    let patch_array = patch.as_array().ok_or(Error::InvalidOrMissingValue(
        "relay overrides must be array",
    ))?;
    let current_array = current_settings
        .as_array()
        .ok_or(Error::InvalidOrMissingValue(
            "existing overrides should be an array",
        ))?;
    let mut new_array = current_array.clone();

    for patch_override in patch_array.iter().cloned() {
        let patch_obj = patch_override
            .as_object()
            .ok_or(Error::InvalidOrMissingValue("override entry"))?;
        let patch_hostname = patch_obj
            .get("hostname")
            .and_then(|hostname| hostname.as_str())
            .ok_or(Error::InvalidOrMissingValue("hostname"))?;

        let existing_obj = new_array.iter_mut().find(|value| {
            value
                .as_object()
                .and_then(|obj| obj.get("hostname"))
                .map(|hostname| hostname.as_str() == Some(patch_hostname))
                .unwrap_or(false)
        });

        match existing_obj {
            Some(existing_val) => {
                // Replace or append to existing values
                match (existing_val, patch_override) {
                    (
                        serde_json::Value::Object(ref mut current),
                        serde_json::Value::Object(ref patch),
                    ) => {
                        for (k, v) in patch {
                            current.insert(k.to_owned(), v.to_owned());
                        }
                    }
                    _ => {
                        return Err(Error::InvalidOrMissingValue(
                            "all override entries must be objects",
                        ));
                    }
                }
            }
            None => new_array.push(patch_override),
        }
    }

    Ok(serde_json::Value::Array(new_array))
}

fn merge_patch_to_value(
    permitted_key: &'static PermittedKey,
    current_value: &mut serde_json::Value,
    patch_value: &serde_json::Value,
    recurse_level: usize,
) -> Result<(), Error> {
    if recurse_level >= RECURSE_LIMIT {
        return Err(Error::RecursionLimit);
    }

    match permitted_key.merge_strategy {
        MergeStrategy::Replace => {
            match (&permitted_key.key_type, current_value, patch_value) {
                // Append or replace keys to objects
                (
                    PermittedKeyValue::Object(sub_permitted),
                    serde_json::Value::Object(ref mut current),
                    serde_json::Value::Object(ref patch),
                ) => {
                    for (k, sub_patch) in patch {
                        let Some((_, sub_permitted)) = sub_permitted
                            .iter()
                            .find(|(permitted_key, _)| k == permitted_key)
                        else {
                            return Err(Error::UnknownOrProhibitedKey(k.to_owned()));
                        };
                        let sub_current = current.entry(k).or_insert(serde_json::Value::Null);
                        merge_patch_to_value(
                            sub_permitted,
                            sub_current,
                            sub_patch,
                            recurse_level + 1,
                        )?;
                    }
                }
                // Totally replace anything else
                (_, current, patch) => {
                    *current = patch.clone();
                }
            }
        }
        MergeStrategy::Custom(merge_function) => {
            *current_value = merge_function(current_value, patch_value)?;
        }
    }

    Ok(())
}

fn validate_patch_value(
    permitted_key: &'static PermittedKey,
    json_value: &serde_json::Value,
    recurse_level: usize,
) -> Result<(), Error> {
    if recurse_level >= RECURSE_LIMIT {
        return Err(Error::RecursionLimit);
    }

    match permitted_key.key_type {
        PermittedKeyValue::Object(subkeys) => {
            let map = json_value.as_object().ok_or(Error::InvalidOrMissingValue(
                "expected JSON object in patch",
            ))?;
            for (k, v) in map.into_iter() {
                // NOTE: We're relying on the parser to shed duplicate keys here.
                // As of this writing, `Map` is implemented using BTreeMap.
                let Some((_, subkey)) =
                    subkeys.iter().find(|(permitted_key, _)| k == permitted_key)
                else {
                    return Err(Error::UnknownOrProhibitedKey(k.to_owned()));
                };
                validate_patch_value(subkey, v, recurse_level + 1)?;
            }
            Ok(())
        }
        PermittedKeyValue::Array(subkey) => {
            let values = json_value
                .as_array()
                .ok_or(Error::InvalidOrMissingValue("expected JSON array in patch"))?;
            for v in values {
                validate_patch_value(subkey, v, recurse_level + 1)?;
            }
            Ok(())
        }
        PermittedKeyValue::Any => Ok(()),
    }
}

#[test]
fn test_permitted_value() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[(
        "key",
        PermittedKey::array(&PermittedKey::object(&[("a", PermittedKey::any())])),
    )]);

    let patch = r#"{"key": [ {"a": "test" } ] }"#;
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap();
}

#[test]
fn test_prohibited_value() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[(
        "key",
        PermittedKey::array(&PermittedKey::object(&[("a", PermittedKey::any())])),
    )]);

    let patch = r#"{"keyx": [] }"#;
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap_err();

    let patch = r#"{"key": { "b": 1 } }"#;
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap_err();
}

#[test]
fn test_merge_append_to_object() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[
        ("test0", PermittedKey::any()),
        ("test1", PermittedKey::any()),
    ]);

    let current = r#"{ "test0": 1 }"#;
    let patch = r#"{ "test1": [] }"#;
    let expected = r#"{ "test0": 1, "test1": [] }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);
}

#[test]
fn test_merge_replace_in_object() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[
        ("test0", PermittedKey::any()),
        (
            "test1",
            PermittedKey::object(&[("a", PermittedKey::any()), ("test0", PermittedKey::any())]),
        ),
    ]);

    let current = r#"{ "test0": 1, "test1": { "a": 1, "test0": [] } }"#;
    let patch = r#"{ "test1": { "test0": [1, 2, 3] } }"#;
    let expected = r#"{ "test0": 1, "test1": { "a": 1, "test0": [1, 2, 3] } }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);
}

#[test]
fn test_overflow() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::array(&PermittedKey::array(
        &PermittedKey::array(&PermittedKey::array(&PermittedKey::array(
            &PermittedKey::array(&PermittedKey::array(&PermittedKey::array(
                &PermittedKey::array(&PermittedKey::array(&PermittedKey::array(
                    &PermittedKey::array(&PermittedKey::array(&PermittedKey::array(
                        &PermittedKey::array(&PermittedKey::array(&PermittedKey::array(
                            &PermittedKey::array(&PermittedKey::any()),
                        ))),
                    ))),
                ))),
            ))),
        ))),
    ));

    let patch = r#"[[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]]"#;
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();

    assert!(matches!(
        validate_patch_value(PERMITTED_SUBKEYS, &patch, 0),
        Err(Error::RecursionLimit)
    ));
}

/// Test whether valid patches from documentation are accepted by the implementation
#[test]
fn test_valid_patch_files() {
    const OVERRIDE_PATCH: &str =
        include_str!("../../../docs/patch-examples/override-relay-ips.json");

    let prev_settings = Settings::default();
    let _ = merge_validate_patch_inner(&prev_settings, OVERRIDE_PATCH)
        .expect("failed to apply relay overrides");
}

#[test]
fn test_patch_export() {
    use mullvad_types::relay_constraints::RelayOverride;

    let mut settings = Settings::default();

    let mut relay_override = RelayOverride::empty("test".to_owned());
    relay_override.ipv4_addr_in = Some("1.2.3.4".parse().unwrap());
    relay_override.ipv6_addr_in = Some("::1".parse().unwrap());
    settings.relay_overrides.push(relay_override);

    let exported = export_settings_inner(&settings).expect("patch export failed");

    let expected = r#"{ "relay_overrides": [ { "hostname": "test", "ipv4_addr_in": "1.2.3.4", "ipv6_addr_in": "::1" } ] }"#;
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    assert_eq!(exported, expected);
}

#[test]
fn test_patch_relay_override() {
    const PERMITTED_SUBKEYS: &PermittedKey = &PermittedKey::object(&[(
        "relay_overrides",
        PermittedKey::array(&PermittedKey::object(&[
            ("hostname", PermittedKey::any()),
            ("ipv4_addr_in", PermittedKey::any()),
            ("ipv6_addr_in", PermittedKey::any()),
        ]))
        .merge_strategy(MergeStrategy::Custom(merge_relay_overrides)),
    )]);

    // If override has no hostname, fail
    //
    let patch = r#"{ "relay_overrides": [ { "invalid": 0 } ] }"#;
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap_err();

    // If there are no overrides, append new override
    //
    let current = r#"{ "other": 1 }"#;
    let patch = r#"{ "relay_overrides": [ { "hostname": "test", "ipv4_addr_in": "1.3.3.7" } ] }"#;
    let expected = r#"{ "other": 1, "relay_overrides": [ { "hostname": "test", "ipv4_addr_in": "1.3.3.7" } ] }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap();
    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);

    // If there are overrides, append new override to existing list
    //
    let current = r#"{ "relay_overrides": [ { "hostname": "test", "ipv4_addr_in": "1.3.3.7" } ] }"#;
    let patch = r#"{ "relay_overrides": [ { "hostname": "new", "ipv4_addr_in": "1.2.3.4" } ] }"#;
    let expected = r#"{ "relay_overrides": [  { "hostname": "test", "ipv4_addr_in": "1.3.3.7" }, { "hostname": "new", "ipv4_addr_in": "1.2.3.4" } ] }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap();
    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);

    // If there are overrides, replace existing overrides but keep rest
    //
    let current = r#"{ "relay_overrides": [  { "hostname": "test", "ipv4_addr_in": "1.3.3.7" }, { "hostname": "test2", "ipv4_addr_in": "1.2.3.4" } ] }"#;
    let patch = r#"{ "relay_overrides": [ { "hostname": "test2", "ipv4_addr_in": "0.0.0.0" }, { "hostname": "test3", "ipv4_addr_in": "192.168.1.1" } ] }"#;
    let expected = r#"{ "relay_overrides": [  { "hostname": "test", "ipv4_addr_in": "1.3.3.7" }, { "hostname": "test2", "ipv4_addr_in": "0.0.0.0" }, { "hostname": "test3", "ipv4_addr_in": "192.168.1.1" } ] }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap();
    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);

    // For same hostname, only update specified overrides
    //
    let current =
        r#"{ "relay_overrides": [  { "hostname": "test", "ipv4_addr_in": "1.3.3.7" } ] }"#;
    let patch = r#"{ "relay_overrides": [  { "hostname": "test", "ipv6_addr_in": "::1" } ] }"#;
    let expected = r#"{ "relay_overrides": [  { "hostname": "test", "ipv4_addr_in": "1.3.3.7", "ipv6_addr_in": "::1" } ] }"#;

    let mut current: serde_json::Value = serde_json::from_str(current).unwrap();
    let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
    let expected: serde_json::Value = serde_json::from_str(expected).unwrap();

    validate_patch_value(PERMITTED_SUBKEYS, &patch, 0).unwrap();
    merge_patch_to_value(PERMITTED_SUBKEYS, &mut current, &patch, 0).unwrap();

    assert_eq!(current, expected);
}
