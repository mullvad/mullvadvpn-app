use regex::Regex;
use std::{env, fs, path::PathBuf, process::Command};

/// How many characters of the git commit that should be added to the version name
/// in dev builds.
const GIT_HASH_DEV_SUFFIX_LEN: usize = 6;

const ANDROID_VERSION_FILE_PATH: &str = "../android/app/build.gradle.kts";
const DESKTOP_VERSION_FILE_PATH: &str = "../gui/package.json";

#[derive(Debug)]
enum Target {
    Android,
    Desktop,
}

impl Target {
    pub fn get() -> Self {
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
    let target = Target::get();
    let mut product_version = parse_current_version_from_file(&target);

    // Compute the expected tag name for the release named `product_version`
    let release_tag = match &target {
        Target::Android => format!("android/{product_version}"),
        Target::Desktop => product_version.clone(),
    };

    // Get the git commit hashes for the latest release and current HEAD
    let product_version_commit_hash = git_rev_parse_commit_hash(&release_tag);
    let current_head_commit_hash =
        git_rev_parse_commit_hash("HEAD").expect("HEAD must have a commit hash");

    // If we are not currently building the release tag, we are on a development build.
    // Adjust product version string accordingly.
    if product_version_commit_hash.as_ref() != Some(&current_head_commit_hash) {
        let hash_suffix = &current_head_commit_hash[..GIT_HASH_DEV_SUFFIX_LEN];
        product_version = format!("{product_version}-dev-{hash_suffix}");
    }

    // TODO: Remove this and all other warnings
    println!("cargo:warning=PRODUCT VERSION {product_version}");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();
}

/// Returns the Mullvad product version from the corresponding metadata files,
/// depending on target platform.
fn parse_current_version_from_file(target: &Target) -> String {
    match target {
        Target::Android => {
            println!("cargo:rerun-if-changed={ANDROID_VERSION_FILE_PATH}");
            get_single_capture_from_file(
                ANDROID_VERSION_FILE_PATH,
                Regex::new("versionName = \"([^\"]*)\"").unwrap(),
            )
        }
        Target::Desktop => {
            println!("cargo:rerun-if-changed={DESKTOP_VERSION_FILE_PATH}");
            let semver_version = get_single_capture_from_file(
                DESKTOP_VERSION_FILE_PATH,
                Regex::new("\"version\": \"([^\"]*)\"").unwrap(),
            );
            semver_version.replacen(".0", "", 1)
        }
    }
}

/// Returns the content of the first capture group in in the single match
/// of the given `regex` over the content of the file at `path`
fn get_single_capture_from_file(path: &str, regex: Regex) -> String {
    let file_content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
    let mut capture = regex.captures_iter(&file_content);
    let regex_match = capture.next().expect("failed to find version capture")[1].to_owned();
    assert!(capture.next().is_none());
    regex_match
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
