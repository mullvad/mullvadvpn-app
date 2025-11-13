//! This module includes all that is needed for the (de)serialization of Mullvad version metadata.
//! This includes ensuring authenticity and integrity of version metadata, and rejecting expired
//! metadata. There are also tools for producing new versions.
//!
//! Fundamentally, a version object is a JSON object with a `signed` key and a `signature` key.
//! `signature` contains a public key and an ed25519 signature of `signed` in canonical JSON form.
//! `signed` also contains an `expires` field, which is a timestamp indicating when the object
//! expires.
//!
//! For the deserializer to succeed in deserializing a file, it must verify that the canonicalized
//! form of `signed` is in fact signed by key/signature in `signature`. It also reads the `expires`
//! and rejects the file if it has expired.

pub mod architecture;
pub mod deserializer;
pub mod installer;
pub mod key;
pub mod release;
pub mod response;
#[cfg(feature = "sign")]
pub mod serializer;

pub use architecture::Architecture;

#[cfg(test)]
mod test {
    use crate::format::release::Release;
    use crate::version::rollout::Rollout;

    #[test]
    fn test_default_rollout_serialize() {
        // rollout should not be serialized if equal to default value
        let serialized = serde_json::to_value(Release {
            version: "2024.1".parse().unwrap(),
            changelog: "".to_owned(),
            installers: vec![],
            rollout: Rollout::complete(),
        })
        .unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "version": "2024.1",
                "changelog": "",
                "installers": [],
            })
        );

        // rollout *should* be serialized if not equal to default value
        let rollout = Rollout::try_from(0.99).unwrap();
        let serialized = serde_json::to_value(Release {
            version: "2024.1".parse().unwrap(),
            changelog: "".to_owned(),
            installers: vec![],
            rollout,
        })
        .unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "version": "2024.1",
                "changelog": "",
                "installers": [],
                "rollout": rollout,
            })
        );
    }
}
