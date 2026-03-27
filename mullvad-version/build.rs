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

#[derive(Debug, Copy, Clone, PartialEq)]
enum Target {
    Android,
    Desktop,
}

impl Target {
    fn current_target() -> Result<Self, String> {
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
        match env::var("CARGO_CFG_TARGET_OS")
            .expect("CARGO_CFG_TARGET_OS should be set")
            .as_str()
        {
            "android" => Ok(Self::Android),
            "linux" | "windows" | "macos" => Ok(Self::Desktop),
            other => Err(other.to_owned()),
        }
    }
}

fn main() {
    // Mark "has_version" as a conditional configuration flag
    println!("cargo::rustc-check-cfg=cfg(has_version)");

    let target = match Target::current_target() {
        Ok(target) => target,
        Err(other) => {
            eprintln!("No version available for target {other}");
            return;
        }
    };

    println!(r#"cargo::rustc-cfg=has_version"#);

    let product_version = get_product_version(target);
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
    let Some((git_dir, git_common_dir)) = git_dirs() else {
        return String::new();
    };
    // Rerun this build script on changes to the git ref that affects the build version.
    // NOTE: This must be kept up to date with the behavior of `git_rev_parse_commit_hash`.
    rerun_if_git_ref_changed(&git_dir, &git_common_dir, release_tag);
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

/// Returns `(git_dir, git_common_dir)` for this repository, or `None` if not in a git repo.
/// In a worktree these differ: `git_dir` is worktree-specific (`HEAD`), `git_common_dir` is
/// shared (`refs/`).
fn git_dirs() -> Option<(PathBuf, PathBuf)> {
    let git_dir = git_output(&["rev-parse", "--git-dir"]).map(PathBuf::from)?;
    let git_common_dir = git_output(&["rev-parse", "--git-common-dir"]).map(PathBuf::from)?;
    Some((git_dir, git_common_dir))
}

/// Trigger rebuild of `mullvad-version` on changing branch (`HEAD`), on changes to the ref of
/// the current branch (`refs/heads/$current_branch`) and on changes to the ref of the current
/// release tag (`refs/tags/$current_release_tag`).
fn rerun_if_git_ref_changed(git_dir: &Path, git_common_dir: &Path, release_tag: &str) {
    // Track HEAD to detect branch switches and detached HEAD changes (commits, rebases).
    // HEAD lives in the worktree-specific git_dir.
    let head_path = git_dir.join("HEAD");
    if head_path.exists() {
        println!("cargo:rerun-if-changed={}", head_path.display());
    }

    // The above check will not cause a rebuild when modifying commits on a currently checked out
    // branch. To catch this, we need to track the `refs/heads/$current_branch` file.
    let current_branch = git_output(&["branch", "--show-current"]).unwrap_or_default();

    // When in 'detached HEAD' state, the output will be empty. However, in that case we already get
    // the ref from `HEAD`, so we can safely skip this part.
    if !current_branch.is_empty() {
        let git_current_branch_ref = git_common_dir
            .join("refs")
            .join("heads")
            .join(current_branch);
        if git_current_branch_ref.exists() {
            println!(
                "cargo:rerun-if-changed={}",
                git_current_branch_ref.display()
            );
        }
    }

    // Since the product version depends on if the build is done on the commit with the
    // corresponding release tag or not, we must track creation of/changes to said tag
    let git_release_tag_ref = git_common_dir.join("refs").join("tags").join(release_tag);
    if git_release_tag_ref.exists() {
        println!("cargo:rerun-if-changed={}", git_release_tag_ref.display());
    };

    // NOTE: As the repository has gotten quite large, you may find the contents of the
    // `refs/heads` and `refs/tags` directories empty. This happens because `git pack-refs`
    // compresses and moves the information into the `packed-refs` file to save storage. We do not
    // have to track this file, however, as any changes to the current branch, 'detached HEAD'
    // state or tags will update the corresponding `refs` file we are tracking, even if it had
    // previously been pruned.
}

/// Returns the commit hash for the commit that `git_ref` is pointing to.
///
/// Returns `None` if the git reference cannot be found.
fn git_rev_parse_commit_hash(git_ref: &str) -> Option<String> {
    git_output(&["rev-parse", &format!("{git_ref}^{{commit}}")])
}

/// Runs a git command with the given arguments and returns the trimmed stdout as a `String`, or
/// `None` if the command fails to execute or exits with a non-zero status.
fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).unwrap().trim().to_owned())
}
