#!/usr/bin/env bash
#
# Prints the versions of the packages currently in our Linux repositories.

set -eu

function usage() {
    echo "Usage: $0 <repository type> <environment>"
    echo ""
    echo "Example usage: $0 rpm production"
    echo
    echo "Arguments:"
    echo "  repository type: deb or rpm"
    echo "  environment: production, staging or dev"
    echo
    echo "Options:"
    echo "  -h | --help		Show this help message and exit."
    exit 1
}

if [[ "$#" == 0 || $1 == "-h" || $1 == "--help" ]]; then
    usage
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "$SCRIPT_DIR/build-linux-repositories-config.sh"

repository="$1"
environment="$2"

case "$environment" in
    "production")
        repository_server_public_url="$PRODUCTION_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    "staging")
        repository_server_public_url="$STAGING_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    "dev")
        repository_server_public_url="$DEV_LINUX_REPOSITORY_PUBLIC_URL"
        ;;
    *)
        echo "Unknown environment. Specify production, staging or dev" >&2
        exit 1
        ;;
esac

if [[ "$repository" == "deb" ]]; then
    podman run --rm -it debian:bookworm-slim sh -c \
        "apt update >/dev/null; \
        apt install -y curl >/dev/null; \
        curl -fsSLo /usr/share/keyrings/mullvad-keyring.asc $repository_server_public_url/deb/mullvad-keyring.asc; \
        echo \"deb [signed-by=/usr/share/keyrings/mullvad-keyring.asc arch=amd64] $repository_server_public_url/deb/stable bookworm main\" > /etc/apt/sources.list.d/mullvad.list; \
        apt update >/dev/null; \
        apt list mullvad-vpn mullvad-browser"
elif [[ "$repository" == "rpm" ]]; then
    podman run --rm -it fedora:latest sh -c \
    "dnf install -y 'dnf-command(config-manager)' >/dev/null; \
    dnf config-manager --add-repo $repository_server_public_url/rpm/stable/mullvad.repo >/dev/null; \
    dnf list --refresh mullvad-vpn mullvad-browser 2>/dev/null | grep -A 1000 'Available Packages'"
else
    echo "Unknown repository type. Specify deb or rpm" >&2
    exit 1
fi
