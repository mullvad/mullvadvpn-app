#!/usr/bin/env bash

# This script will build, sign and publish the Mullvad VPN app build image(s)
# for either desktop or android, depending on the first argument.
# Please see `README.md` for setup instructions *before* running this script

set -eu

CONTAINER_SIGNING_KEY_FINGERPRINT=1E551687D67F5FD820BEF2C4D7C17F87A0D3D215

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

source scripts/utils/log

tag="$(git rev-parse --short HEAD)"

case ${1-:""} in
    desktop)
        container_name="ghcr.io/mullvad/mullvadvpn-app-build"
        container_context_dir="."
        container_image_tag_path="dist-assets/desktop-container-image-tag.txt"
    ;;
    android)
        container_name="ghcr.io/mullvad/mullvadvpn-app-build-android"
        container_context_dir="android/docker/"
        container_image_tag_path="dist-assets/android-container-image-tag.txt"
    ;;
    *)
        log_error "Invalid platform. Specify 'desktop' or 'android' as first argument"
        exit 1
esac

log_header "Building $container_name tagged as '$tag' and 'latest'"
podman build "$container_context_dir" --no-cache \
    -t "$container_name:$tag" \
    -t "$container_name:latest"

# Temporary directory to store image digest and signature in.
# This is a hack since two consecutive `podman push` seems
# to overwrite the signatures. We want to keep the signature
# for both the 'latest' and '$tag' tags.
tmp_signature_dir=$(mktemp -d)

function delete_tmp_signature_dir {
    rm -rf "$tmp_signature_dir"
}
trap 'delete_tmp_signature_dir' EXIT

log_header "Pushing $container_name:latest"
podman push "$container_name:latest" \
    --sign-by $CONTAINER_SIGNING_KEY_FINGERPRINT \
    --digestfile "$tmp_signature_dir/digest_latest"

digest=$(cat "$tmp_signature_dir/digest_latest")
log_success "Pushed image with digest $digest"
# Backup the signature so we can restore it after the second podman push later
signature_dir="build-containers/sigstore/mullvad/mullvadvpn-app-build@${digest/:/=}"
if [[ -f "$signature_dir/signature-2" ]]; then
    log_error "Did not expect $signature_dir/signature-2 to exist"
    exit 1
fi
cp "$signature_dir/signature-1" "$tmp_signature_dir/signature-2"

log_header "Pushing $container_name:$tag"
podman push "$container_name:$tag" \
    --sign-by $CONTAINER_SIGNING_KEY_FINGERPRINT \
    --digestfile "$tmp_signature_dir/digest_$tag"

if ! cmp -s "$tmp_signature_dir/digest_latest" "$tmp_signature_dir/digest_$tag"; then
    log_error "Digests differ between 'latest' and '$tag' pushes"
    exit 1
fi

cp "$tmp_signature_dir/signature-2" "$signature_dir/"

log_info "Storing container tag to $container_image_tag_path"
echo "$tag" > "$container_image_tag_path"

log_header "Commiting signatures and new tag name to git"
git add "$container_image_tag_path" "$signature_dir"
GPG_TTY=$(tty) git commit -S -m "Updating build container for $1 to $tag"

log_success "***********************"
log_success ""
log_success "Done building and pushing $container_name with tags '$tag' and 'latest'"
log_success "Make sure to push the changes to git"
log_success ""
log_success "***********************"
