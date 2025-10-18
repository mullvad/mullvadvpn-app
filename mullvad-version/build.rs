use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{Context, bail};
use gix::Repository;

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
    fn current_target() -> anyhow::Result<Self> {
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
        match env::var("CARGO_CFG_TARGET_OS")
            .context("CARGO_CFG_TARGET_OS should be set")?
            .as_str()
        {
            "android" => Ok(Self::Android),
            "linux" | "windows" | "macos" => Ok(Self::Desktop),
            other => bail!(other.to_owned()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let repository = gix::discover(".")?;
    // Mark "has_version" as a conditional configuration flag
    println!("cargo::rustc-check-cfg=cfg(has_version)");

    let target = match Target::current_target() {
        Ok(target) => target,
        Err(other) => {
            bail!("No version available for target {other}");
        }
    };

    println!(r#"cargo::rustc-cfg=has_version"#);

    let product_version = get_product_version(&repository, target)?;
    let android_product_version = get_product_version(&repository, Target::Android)?;

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").context("OUT_DIR should be set")?);
    fs::write(out_dir.join("product-version.txt"), product_version)?;
    fs::write(
        out_dir.join("android-version-name.txt"),
        android_product_version,
    )?;

    Ok(())
}

/// Computes the Mullvad product version using the latest release on the given platform and the git
/// hash pointed to by `HEAD`. Also triggers a rebuild of this crate when the information becomes
/// outdated.
fn get_product_version(repository: &Repository, target: Target) -> anyhow::Result<String> {
    let version_file_path = match target {
        Target::Android => ANDROID_VERSION_FILE_PATH,
        Target::Desktop => DESKTOP_VERSION_FILE_PATH,
    };
    println!("cargo:rerun-if-changed={version_file_path}");

    let release_version = fs::read_to_string(version_file_path)
        .context("Failed to read {version_file_path}")?
        .trim()
        .to_owned();

    // Compute the expected tag name for the release named `product_version`
    let release_tag = match target {
        Target::Android => format!("android/{release_version}"),
        Target::Desktop => release_version.clone(),
    };

    Ok(format!(
        "{release_version}{}",
        get_suffix(repository, &release_tag)?
    ))
}

/// Returns the suffix for the current build. If the build is done on a git tag named
/// `product_version` or a git repository cannot be found, the suffix is empty. Otherwise,
/// `-dev-$hash` is appended to the release version.
fn get_suffix(repository: &Repository, release_tag: &str) -> anyhow::Result<String> {
    // Rerun this build script on changes to the git ref that affects the build version.
    // NOTE: This must be kept up to date with the behavior of `git_rev_parse_commit_hash`.
    rerun_if_git_ref_changed(repository, release_tag)?;
    let head_commit_hash = git_rev_parse_commit_hash("HEAD")
        .context("Failed to run `git rev-parse HEAD^{{commit}}`")?;
    let product_version_commit_hash = git_rev_parse_commit_hash(release_tag);

    // If we are currently building the release tag, there is no dev suffix
    if Some(&head_commit_hash) == product_version_commit_hash.as_ref() {
        Ok(String::new())
    } else {
        Ok(format!(
            "-dev-{}",
            &head_commit_hash[..GIT_HASH_DEV_SUFFIX_LEN]
        ))
    }
}

/// Trigger rebuild of `mullvad-version` on changing branch (`.git/HEAD`), on changes to the ref of
/// the current branch (`.git/refs/heads/$current_branch`) and on changes to the ref of the current
/// release tag (`.git/refs/tags/$current_release_tag`). // TODO: This is not true anymore
fn rerun_if_git_ref_changed(repository: &Repository, _release_tag: &str) -> anyhow::Result<()> {
    // The `.git/HEAD` file contains the position of the current head. If in 'detached HEAD' state,
    // this will be the ref of the current commit. If on a branch it will just point to it, e.g.
    // `ref: refs/heads/main`. Tracking changes to this file will tell us if we change branch, or
    // modify the current detached HEAD state (e.g. committing or rebasing).
    // HACK: Can we do this better ??
    println!(
        "cargo:rerun-if-changed={}",
        repository.path().join("HEAD").display()
    );

    // NOTE: As the repository has gotten quite large, you may find the contents of the
    // `.git/refs/heads` and `.git/refs/tags` empty. This happens because `git pack-refs` compresses
    // and moves the information into the `.git/packed-refs` file to save storage. We do not have to
    // track this file, however, as any changes to the current branch, 'detached HEAD' state
    // or tags will update the corresponding `.git/refs` file we are tracking, even if it had
    // previously been pruned.
    match repository.head()?.kind {
        gix::head::Kind::Symbolic(reference) => {
            if let Some(category) = reference.name.category() {
                match category {
                    // Since the product version depends on if the build is done on the commit with the
                    // corresponding release tag or not, we must track creation of/changes to said tag
                    gix::refs::Category::Tag => {
                        let tag_ref_path = repository
                            .path()
                            .join("refs")
                            .join("tags")
                            .join(reference.name.shorten().to_string());
                        println!("cargo:rerun-if-changed={}", tag_ref_path.display())
                    }
                    // The above check will not cause a rebuild when modifying commits on a currently checked out
                    // branch. To catch this, we need to track the `.git/refs/heads/$current_branch` file.
                    gix::refs::Category::LocalBranch => {
                        let branch_ref_path = repository
                            .path()
                            .join("refs")
                            .join("heads")
                            .join(reference.name.shorten().to_string());
                        println!("cargo:rerun-if-changed={}", branch_ref_path.display())
                    }
                    // skip
                    _ => (),
                }
            }
        }
        gix::head::Kind::Detached { .. } => (),
        gix::head::Kind::Unborn(_) => bail!("New repository"),
    }
    Ok(())
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
