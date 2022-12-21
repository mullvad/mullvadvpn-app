use std::{env, fs, path::PathBuf, process::Command};

/// How many characters of the git commit that should be added to the version name
/// in dev builds.
const GIT_HASH_DEV_SUFFIX_LEN: usize = 6;

const ANDROID_VERSION_FILE_PATH: &str = "../dist-assets/android-product-version.txt";
const DESKTOP_VERSION_FILE_PATH: &str = "../dist-assets/desktop-product-version.txt";

#[derive(Debug, Copy, Clone)]
enum Target {
    Android,
    Desktop,
}

impl Target {
    pub fn current_target() -> Self {
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
        match env::var("CARGO_CFG_TARGET_OS")
            .expect("CARGO_CFG_TARGET_OS should be set")
            .as_str()
        {
            "android" => Self::Android,
            "linux" | "windows" | "macos" => Self::Desktop,
            target_os => panic!("Unsupported target OS: {target_os}"),
        }
    }
}

fn main() {
    let product_version = get_product_version(Target::current_target());
    let android_product_version = get_product_version(Target::Android);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("product-version.txt"), product_version).unwrap();
    fs::write(
        out_dir.join("android-product-version.txt"),
        android_product_version,
    )
    .unwrap();
}

/// Returns the Mullvad product version from the corresponding metadata files,
/// depending on target platform.
fn get_product_version(target: Target) -> String {
    let version_file_path = match target {
        Target::Android => ANDROID_VERSION_FILE_PATH,
        Target::Desktop => DESKTOP_VERSION_FILE_PATH,
    };
    println!("cargo:rerun-if-changed={version_file_path}");
    let version = fs::read_to_string(version_file_path)
        .unwrap_or_else(|_| panic!("Failed to read {version_file_path}"))
        .trim()
        .to_owned();

    let dev_suffix = get_dev_suffix(target, &version);

    format!("{version}{dev_suffix}")
}

fn get_dev_suffix(target: Target, product_version: &str) -> String {
    // Compute the expected tag name for the release named `product_version`
    let release_tag = match target {
        Target::Android => format!("android/{product_version}"),
        Target::Desktop => product_version.to_owned(),
    };

    // Get the git commit hashes for the latest release and current HEAD
    let product_version_commit_hash = git_rev_parse_commit_hash(&release_tag);
    let current_head_commit_hash =
        git_rev_parse_commit_hash("HEAD").expect("HEAD must have a commit hash");

    // If we are not currently building the release tag, we are on a development build.
    // Adjust product version string accordingly.
    if product_version_commit_hash.as_ref() != Some(&current_head_commit_hash) {
        let hash_suffix = &current_head_commit_hash[..GIT_HASH_DEV_SUFFIX_LEN];
        format!("-dev-{hash_suffix}")
    } else {
        "".to_owned()
    }
}

/// Returns the commit hash for the commit that `git_ref` is pointing to
fn git_rev_parse_commit_hash(git_ref: &str) -> Option<String> {
    // This is a very blunt way of making sure we run again if a tag is added or removed.
    println!("cargo:rerun-if-changed=.git");

    let output = Command::new("git")
        .arg("rev-parse")
        .arg(format!("{git_ref}^{{commit}}"))
        .output()
        .expect("Not able to run git");
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).unwrap().trim().to_owned())
}
