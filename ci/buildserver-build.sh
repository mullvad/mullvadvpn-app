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
# * Put the comodo.pfx certificate in the same folder as this script
# * Create sign.bat in the same folder as this script, with the content:
#     signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /d "Mullvad VPN" /du https://github.com/mullvad/mullvadvpn-app#readme /f comodo.pfx /p <PASSWORD TO comodo.pfx> "%1"

set -eu
shopt -s nullglob

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="$SCRIPT_DIR/mullvadvpn-app"
LAST_BUILT_DIR="$SCRIPT_DIR/last-built"
UPLOAD_DIR="/home/upload/upload"

BRANCHES_TO_BUILD=("origin/master")

case "$(uname -s)" in
  Darwin*)
    if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
      read -sp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
      echo ""
      export CSC_KEY_PASSWORD
    fi
    ;;
  MINGW*|MSYS_NT*)
    if [[ -z ${CERT_PASSPHRASE-} ]]; then
      read -sp "CERT_PASSPHRASE = " CERT_PASSPHRASE
      echo ""
      export CERT_PASSPHRASE
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

# Sign the Windows app. We try multiple times because it can randomly fail to
# contact the timestamp server.
# signtool must be called via a bat file, I cant make it work any other way :(
sign_win() {
  echo "Signing Windows Mullvad VPN installer"
  echo 'signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /d "Mullvad VPN" /du https://github.com/mullvad/mullvadvpn-app#readme /f "%1" /p "%2" "%3"' > "$SCRIPT_DIR/sign.bat"
  for _ in {0..3}; do
    sleep 1
    $SCRIPT_DIR/sign.bat $SCRIPT_DIR/comodo.pfx "$CERT_PASSPHRASE" dist/MullvadVPN-*.exe && return 0
  done
  return 1
}

upload() {
  current_hash=$1
  for f in MullvadVPN-*.{deb,rpm,exe,pkg,apk}; do
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

build_ref() {
  ref=$1
  tag_or_branch=$2

  ref_filename="${ref/\//_}"
  last_built_hash="$(cat $LAST_BUILT_DIR/$ref_filename || echo "N/A")"
  current_hash="$(git rev-parse $ref)"

  if [ "$last_built_hash" == "$current_hash" ]; then
    # Same commit as last time this ref was built, not building
    return 0
  fi

  echo ""
  echo "[#] $ref: $last_built_hash->$current_hash, building new packages."

  if [ "$tag_or_branch" == "tag" ] && ! git verify-tag $ref; then
    echo "!!!"
    echo "[#] $ref is a tag, but it failed GPG verification!"
    echo "!!!"
    sleep 60
    return 0
  elif [ "$tag_or_branch" == "branch" ] && ! git verify-commit $current_hash; then
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

  # Make sure we have the latest Rust toolchain before the build
  rustup update

  ./build.sh || return 0
  case "$(uname -s)" in
    MINGW*|MSYS_NT*)
      sign_win || return 0
      echo "Packaging all PDB files..."
      find ./windows/ -iname "*.pdb" | tar -cJf $SCRIPT_DIR/pdb/$current_hash.tar.xz -T -
      ;;
    Linux*)
      echo "Building Android APK"
      ./build-apk.sh || return 0
      ;;
  esac

  (cd dist/ && upload $current_hash) || return 0
  echo "$current_hash" > "$LAST_BUILT_DIR/$ref_filename"
  echo "Successfully finished build at $(date)"
}

cd "$BUILD_DIR"

while true; do
  # Delete all tags. So when fetching we only get the ones existing on the remote
  git tag | xargs git tag -d > /dev/null

  git fetch --prune --tags 2> /dev/null || continue
  tags=( $(git tag) )

  for tag in "${tags[@]}"; do
    build_ref $tag tag
  done

  for branch in "${BRANCHES_TO_BUILD[@]}"; do
    build_ref $branch branch
  done

  sleep 240
done
