#!/usr/bin/env bash

# # Setup instructions before this script will work
#
# * Follow the instructions in ../README.md
# * Import and trust the GPG keys of everyone who the build server should trust code from
# * Set up an entry in `~/.ssh/config` for app-build-linux
# * Add the build servers public ssh key to the upload account on app-build-linux
#
# ## Windows
#
# * Add signtool.exe to your PATH: C:\Program Files (x86)\Windows Kits\10\bin\10.0.16299.0\x64

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
PDB_DIR="$SCRIPT_DIR/pdb"
UPLOAD_DIR="/home/upload/upload"

BRANCHES_TO_BUILD=("origin/master")

case "$(uname -s)" in
  Darwin*|MINGW*|MSYS_NT*)
    if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
      read -sp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
      echo ""
      export CSC_KEY_PASSWORD
    fi
    ;;
esac

# Uploads whatever matches the first argument to the Linux build server
upload_sftp() {
  echo "Uploading Mullvad VPN installers to app-build-linux:upload/"
  sftp app-build-linux <<EOF
cd upload
put $1
bye
EOF
}

upload() {
  for f in MullvadVPN-*.{deb,rpm,exe,pkg,apk,aab}; do
    sha256sum "$f" > "$f.sha256"
    case "$(uname -s)" in
      # Linux is both the build and upload server. Just move directly to target dir
      Linux*)
        mv "$f" "$f.sha256" "$UPLOAD_DIR/"
        ;;
      # Other platforms need to transfer their artifacts to the Linux build machine.
      Darwin*|MINGW*|MSYS_NT*)
        upload_sftp "$f" || return 1
        upload_sftp "$f.sha256" || return 1
        ;;
    esac
  done
}

upload_pdb() {
  current_hash=$1
  f="pdb-$current_hash.tar.xz"

  sha256sum "$f" > "$f.sha256"
  upload_sftp "$f" || return 1
  upload_sftp "$f.sha256" || return 1
}

build_ref() {
  ref=$1
  tag=${2:-""}

  current_hash="$(git rev-parse $ref^{commit})"
  if [ -f "$LAST_BUILT_DIR/$current_hash" ]; then
    # This commit has already been built
    return 0
  fi

  echo ""
  echo "[#] $ref: $current_hash, building new packages."

  if [[ $ref == "refs/tags/"* ]] && ! git verify-tag $ref; then
    echo "!!!"
    echo "[#] $ref is a tag, but it failed GPG verification!"
    echo "!!!"
    sleep 60
    return 0
  elif [[ $ref == "refs/remotes/"* ]] && ! git verify-commit $current_hash; then
    echo "!!!"
    echo "[#] $ref is a branch, but it failed GPG verification!"
    echo "!!!"
    sleep 60
    return 0
  fi

  # Clean our working dir and check out the code we want to build
  rm -r dist/ 2&>/dev/null || true
  git reset --hard
  git checkout $ref
  git submodule update
  git clean -df

  # Make sure we have the latest Rust and Node toolchains before the build
  rustup update

  BUILD_ARGS=(--optimize --sign)
  if [[ "$(uname -s)" == "Darwin" ]]; then
    BUILD_ARGS+=(--universal)
  fi
  ./build.sh "${BUILD_ARGS[@]}" || return 0

  case "$(uname -s)" in
    MINGW*|MSYS_NT*)
      echo "Packaging all PDB files..."
      find ./windows/ \
        ./target/release/mullvad-daemon.pdb \
        ./target/release/mullvad.pdb \
        ./target/release/mullvad-problem-report.pdb \
        -iname "*.pdb" | tar -cJf $PDB_DIR/pdb-$current_hash.tar.xz -T -
      ;;
    Linux*)
      echo "Building Android APK"
      ./build-apk.sh --app-bundle || return 0
      ;;
  esac

  # If there is a tag for this commit then we append that to the produced artifacts
  # A version suffix should only be created if there is a tag for this commit and it is not a release build
  if [[ -n "$tag" ]]; then
      # Remove disallowed version characters from the tag
      version_suffix="+${tag//[^0-9a-z_-]/}"
      # Will only match paths that include *-dev-* which means release builds will not be included
      for artifact_path in dist/MullvadVPN-*-dev-*.{deb,rpm,exe,pkg,apk,aab}; do
          extension="${artifact_path##*.}"
          filename="${artifact_path%.*}"
          mv "$artifact_path" "${filename}${version_suffix}.${extension}"
      done
  fi

  (cd dist/ && upload) || return 0
  case "$(uname -s)" in
    MINGW*|MSYS_NT*)
      (cd "$PDB_DIR" && upload_pdb $current_hash) || return 0
    ;;
  esac
  touch "$LAST_BUILT_DIR/$current_hash"
  echo "Successfully finished build at $(date)"
}

cd "$BUILD_DIR"

while true; do
  # Delete all tags. So when fetching we only get the ones existing on the remote
  git tag | xargs git tag -d > /dev/null

  git fetch --prune --tags 2> /dev/null || continue
  tags=( $(git tag) )

  for tag in "${tags[@]}"; do
    build_ref "refs/tags/$tag" "$tag"
  done

  for branch in "${BRANCHES_TO_BUILD[@]}"; do
    build_ref "refs/remotes/$branch"
  done

  sleep 240
done
