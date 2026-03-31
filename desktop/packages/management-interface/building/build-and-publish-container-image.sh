#!/usr/bin/env bash

# This script will build, sign and publish the Mullvad VPN app build node gRPC image.
# Please see `README.md` in parent folder for setup instructions *before* running this script.

set -eu

CONTAINER_SIGNING_KEY_FINGERPRINT=${CONTAINER_SIGNING_KEY_FINGERPRINT:-"1E551687D67F5FD820BEF2C4D7C17F87A0D3D215"}
REGISTRY_HOST=${REGISTRY_HOST:-"ghcr.io"}
REGISTRY_ORG=${REGISTRY_ORG:-"mullvad"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$( cd "$SCRIPT_DIR/../../../.." && pwd )"
DESKTOP_DIR="$( cd "$SCRIPT_DIR/../../.." && pwd )"
cd "$DESKTOP_DIR"

source "$ROOT_DIR/scripts/utils/log"

tag="$(git rev-parse --short HEAD)"
container_name="mullvadvpn-app-build-node-grpc-bindings"
containerfile_path="$SCRIPT_DIR/Dockerfile" # TODO: Conditionally use Dockerfile.arm64
container_context_dir="$DESKTOP_DIR"
full_container_name="$REGISTRY_HOST/$REGISTRY_ORG/$container_name"

log_header "Building $full_container_name tagged as '$tag' and 'latest'"
podman build -f "$containerfile_path" "$container_context_dir" --no-cache \
    -t "$full_container_name:$tag" \
    -t "$full_container_name:latest"

# Temporary directory to store image digest and signature in.
# This is a hack since two consecutive `podman push` seems
# to overwrite the signatures. We want to keep the signature
# for both the 'latest' and '$tag' tags.
tmp_signature_dir=$(mktemp -d)

function delete_tmp_signature_dir {
    rm -rf "$tmp_signature_dir"
}
trap 'delete_tmp_signature_dir' EXIT

log_header "Pushing $full_container_name:latest"
podman push "$full_container_name:latest" \
    --sign-by "$CONTAINER_SIGNING_KEY_FINGERPRINT" \
    --digestfile "$tmp_signature_dir/digest_latest"

digest=$(cat "$tmp_signature_dir/digest_latest")
log_success "Pushed image with digest $digest"

# Backup the signature so we can restore it after the second podman push later
signature_dir="$ROOT_DIR/building/sigstore/$REGISTRY_ORG/$container_name@${digest/:/=}"
if [[ -f "$signature_dir/signature-2" ]]; then
    log_error "Did not expect $signature_dir/signature-2 to exist"
    exit 1
fi
cp "$signature_dir/signature-1" "$tmp_signature_dir/signature-2"

log_header "Pushing $full_container_name:$tag"
podman push "$full_container_name:$tag" \
    --sign-by "$CONTAINER_SIGNING_KEY_FINGERPRINT" \
    --digestfile "$tmp_signature_dir/digest_$tag"

if ! cmp -s "$tmp_signature_dir/digest_latest" "$tmp_signature_dir/digest_$tag"; then
    log_error "Digests differ between 'latest' and '$tag' pushes"
    exit 1
fi

cp "$tmp_signature_dir/signature-2" "$signature_dir/"

log_header "Committing container sigstore signatures"
git add "$signature_dir"
GPG_TTY=$(tty) git commit -S -m "Add container signature for $container_name:$tag"

log_success "***********************"
log_success ""
log_success "Done building and pushing $full_container_name with tags '$tag' and 'latest'"
log_success "Make sure to push the changes to git"
log_success ""
log_success "***********************"
