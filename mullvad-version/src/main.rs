#[cfg(has_version)]
mod inner {
    use mullvad_version::{PreStableType, Version};
    use std::env::VarError;
    use std::{env, process::exit};

    const ANDROID_VERSION: &str =
        include_str!(concat!(env!("OUT_DIR"), "/android-version-name.txt"));

    pub fn main() {
        let android_version_env = env::var("ANDROID_VERSION");
        if matches!(android_version_env, Err(VarError::NotUnicode(_))) {
            eprintln!("ANDROID_VERSION is not valid unicode.");
            exit(1);
        }
        let android_version = android_version_env.unwrap_or(ANDROID_VERSION.to_string());

        let command = env::args().nth(1);
        match command.as_deref() {
            None => println!("{}", mullvad_version::VERSION),
            Some("semver") => println!("{}", to_semver(mullvad_version::VERSION)),
            Some("version.h") => println!("{}", to_windows_h_format(mullvad_version::VERSION)),
            Some("versionName") => println!("{android_version}"),
            Some("versionCode") => println!("{}", to_android_version_code(&android_version)),
            Some(command) => {
                eprintln!("Unknown command: {command}");
                exit(1);
            }
        }
    }

    /// Takes a version without a patch number and adds the patch (set to zero).
    ///
    /// Converts `x.y[-z]` into `x.y.0[-z]` to make the version semver compatible.
    fn to_semver(version: &str) -> String {
        let mut parts = version.splitn(2, '-');

        let version = parts.next().expect("Year component");
        let remainder = parts.next().map(|s| format!("-{s}")).unwrap_or_default();
        assert_eq!(parts.next(), None);

        format!("{version}.0{remainder}")
    }

    /// Takes a version in the normal Mullvad VPN app version format and returns the Android
    /// `versionCode` formatted version.
    ///
    /// The format of the code is:                    YYVVXZZZ
    ///   Last two digits of the year (major)---------^^
    ///   Incrementing version (minor)------------------^^
    ///   Build type (0=alpha, 1=beta, 9=stable/dev)------^
    ///   Build number (000 if stable/dev)-----------------^^^
    ///
    /// # Examples
    ///
    /// Version: 2021.1-alpha1
    /// versionCode: 21010001
    ///
    /// Version: 2021.34-beta5
    /// versionCode: 21341005
    ///
    /// Version: 2021.34
    /// versionCode: 21349000
    ///
    /// Version: 2021.34-dev
    /// versionCode: 21349000
    fn to_android_version_code(version: &str) -> String {
        let version: Version = version.parse().unwrap();

        let (build_type, build_number) = if version.dev.is_some() {
            ("9", "000".to_string())
        } else {
            match &version.pre_stable {
                Some(PreStableType::Alpha(v)) => ("0", v.to_string()),
                Some(PreStableType::Beta(v)) => ("1", v.to_string()),
                // Stable version
                None => ("9", "000".to_string()),
            }
        };

        let year_last_two_digits = version.year % 100;

        format!(
            "{}{:0>2}{}{:0>3}",
            year_last_two_digits, version.incremental, build_type, build_number,
        )
    }

    fn to_windows_h_format(version_str: &str) -> String {
        let version = version_str.parse().unwrap();

        let Version {
            year, incremental, ..
        } = version;

        format!(
            "#define MAJOR_VERSION {year}
    #define MINOR_VERSION {incremental}
    #define PATCH_VERSION 0
    #define PRODUCT_VERSION \"{version_str}\""
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_version_code() {
            assert_eq!("21349000", to_android_version_code("2021.34"));
        }

        #[test]
        fn test_version_code_alpha() {
            assert_eq!("21010001", to_android_version_code("2021.1-alpha1"));
        }

        #[test]
        fn test_version_code_beta() {
            assert_eq!("21341005", to_android_version_code("2021.34-beta5"));
        }

        #[test]
        fn test_version_code_dev() {
            assert_eq!("21349000", to_android_version_code("2021.34-dev-be846a5f0"));
        }

        #[test]
        fn test_windows_version_h() {
            let version_h = to_windows_h_format("2025.4-beta2-dev-abcdef");
            let expected_version_h = "#define MAJOR_VERSION 2025
    #define MINOR_VERSION 4
    #define PATCH_VERSION 0
    #define PRODUCT_VERSION \"2025.4-beta2-dev-abcdef\"";
            assert_eq!(expected_version_h, version_h);
        }
    }
}

#[cfg(not(has_version))]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main();
}
