use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// How many characters of the git commit that should be added to the version name
/// in dev builds.
const GIT_HASH_DEV_SUFFIX_LEN: usize = 6;

const ANDROID_VERSION_FILE_PATH: &str = "../dist-assets/android-version-name.txt";
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
        out_dir.join("android-version-name.txt"),
        android_product_version,
    )
    .unwrap();
}

/// Computes the Mullvad product version using the latest release on the given platform and the git
/// hash pointed to by `HEAD`. Also triggers a rebuild of this crate when the information becomes
/// outdated.
fn get_product_version(target: Target) -> String {
    let version_file_path = match target {
        Target::Android => ANDROID_VERSION_FILE_PATH,
        Target::Desktop => DESKTOP_VERSION_FILE_PATH,
    };
    println!("cargo:rerun-if-changed={version_file_path}");

    let release_version = fs::read_to_string(version_file_path)
        .unwrap_or_else(|_| panic!("Failed to read {version_file_path}"))
        .trim()
        .to_owned();

    // Compute the expected tag name for the release named `product_version`
    let release_tag = match target {
        Target::Android => format!("android/{release_version}"),
        Target::Desktop => release_version.clone(),
    };

    format!("{release_version}{}", get_suffix(&release_tag))
}

/// Returns the suffix for the current build. If the build is done on a git tag named
/// `product_version` or a git repository cannot be found, the suffix is empty. Otherwise,
/// `-dev-$hash` is appended to the release version.
fn get_suffix(release_tag: &str) -> String {
    if !valid_git_repo() {
        return String::new();
    };
    // Rerun this build script on changes to the git ref that affects the build version.
    // NOTE: This must be kept up to date with the behavior of `git_rev_parse_commit_hash`.
    rerun_if_git_ref_changed(release_tag);
    let head_commit_hash =
        git_rev_parse_commit_hash("HEAD").expect("Failed to run `git rev-parse HEAD^{{commit}}`");
    let product_version_commit_hash = git_rev_parse_commit_hash(release_tag);

    // If we are currently building the release tag, there is no dev suffix
    if Some(&head_commit_hash) == product_version_commit_hash.as_ref() {
        String::new()
    } else {
        format!("-dev-{}", &head_commit_hash[..GIT_HASH_DEV_SUFFIX_LEN])
    }
}

fn valid_git_repo() -> bool {
    matches!(Command::new("git").arg("status").status(), Ok(status) if status.success())
}

/// Trigger rebuild of `mullvad-version` on changing branch (`.git/HEAD`), on changes to the ref of
/// the current branch (`.git/refs/heads/$current_branch`) and on changes to the ref of the current
/// release tag (`.git/refs/tags/$current_release_tag`).
///
/// Returns an error if not in a git repository, or the git binary is not in `PATH`.
fn rerun_if_git_ref_changed(release_tag: &str) {
    let git_dir = Path::new("..").join(".git");

    // The `.git/HEAD` file contains the position of the current head. If in 'detached HEAD' state,
    // this will be the ref of the current commit. If on a branch it will just point to it, e.g.
    // `ref: refs/heads/main`. Tracking changes to this file will tell us if we change branch, or
    // modify the current detached HEAD state (e.g. committing or rebasing).
    let head_path = git_dir.join("HEAD");
    if head_path.exists() {
        println!("cargo:rerun-if-changed={}", head_path.display());
    }

    // The above check will not cause a rebuild when modifying commits on a currently checked out
    // branch. To catch this, we need to track the `.git/refs/heads/$current_branch` file.
    let output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .expect("Failed to execute `git branch --show-current`");

    let current_branch = String::from_utf8(output.stdout).unwrap();
    let current_branch = current_branch.trim();

    // When in 'detached HEAD' state, the output will be empty. However, in that case we already get
    // the ref from `.git/HEAD`, so we can safely skip this part.
    if !current_branch.is_empty() {
        let git_current_branch_ref = git_dir.join("refs").join("heads").join(current_branch);
        if git_current_branch_ref.exists() {
            println!(
                "cargo:rerun-if-changed={}",
                git_current_branch_ref.display()
            );
        }
    }

    // Since the product version depends on if the build is done on the commit with the
    // corresponding release tag or not, we must track creation of/changes to said tag
    let git_release_tag_ref = git_dir.join("refs").join("tags").join(release_tag);
    if git_release_tag_ref.exists() {
        println!("cargo:rerun-if-changed={}", git_release_tag_ref.display());
    };

    // NOTE: As the repository has gotten quite large, you may find the contents of the
    // `.git/refs/heads` and `.git/refs/tags` empty. This happens because `git pack-refs` compresses
    // and moves the information into the `.git/packed-refs` file to save storage. We do not have to
    // track this file, however, as any changes to the current branch, 'detached HEAD' state
    // or tags will update the corresponding `.git/refs` file we are tracking, even if it had
    // previously been pruned.
}

/// Returns the commit hash for the commit that `git_ref` is pointing to.
///
/// Returns `None` if the git reference cannot be found.
fn git_rev_parse_commit_hash(git_ref: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg(format!("{git_ref}^{{commit}}"))
        .output()
        .expect("Failed to run `git rev-parse`");
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).unwrap().trim().to_owned())
}
