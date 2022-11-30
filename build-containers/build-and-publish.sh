#!/usr/bin/env bash

# This script will build, sign and publish the Mullvad VPN app build image(s)
# for either desktop or android, depending on the first argument.
# Please see `README.md` for setup instructions *before* running this script

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

source scripts/utils/log

tag="$(git rev-parse --short HEAD)"

case ${1-:""} in
    desktop)
        container_name="ghcr.io/mullvad/mullvadvpn-app-build:$tag"
        container_context_dir="."
        container_image_tag_path="dist-assets/desktop-container-image-tag.txt"
    ;;
    android)
        container_name="ghcr.io/mullvad/mullvadvpn-app-build-android:$tag"
        container_context_dir="android/docker/"
        container_image_tag_path="dist-assets/android-container-image-tag.txt"
    ;;
    *)
        log_error "Invalid platform. Specify \"desktop\" or \"android\" as first argument"
        exit 1
esac

log_header "Building container $container_name"
podman build "$container_context_dir" -t "$container_name"

log_header "Pushing container $container_name"
podman push --sign-by 1E551687D67F5FD820BEF2C4D7C17F87A0D3D215 "$container_name"

log_info "Storing container tag to $container_image_tag_path"
echo "$tag" > "$container_image_tag_path"

log_header "Commiting signatures and new tag name to git"
git add "$container_image_tag_path" build-containers/sigstore/mullvad/mullvadvpn-app-build*
GPG_TTY=$(tty) git commit -S -m "Updating build container for $1 to $tag"

log_success "***********************"
log_success ""
log_success "Done building and pushing $container_name"
log_success "Make sure to push the changes to git"
log_success ""
log_success "***********************"
