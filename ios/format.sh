#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 [format|lint] [additional swift-format options]"
    exit 1
}

if [[ $# -lt 1 ]]; then
    usage
fi

command=$1
shift

case "$command" in
    format|lint)
        ;;
    *)
        usage
        ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

included_folders=(
    "MullvadLogging"
    "MullvadMockData"
    "MullvadPostQuantum"
    "MullvadREST"
    "MullvadRESTTests"
    "MullvadRustRuntime"
    "MullvadRustRuntimeTests"
    "MullvadSettings"
    "MullvadTypes"
    "MullvadVPN"
    "MullvadVPNTests"
    "MullvadVPNUITests"
    "Operations"
    "OperationsTests"
    "PacketTunnel"
    "PacketTunnelCore"
    "PacketTunnelCoreTests"
    "Routing"
    "RoutingTests"
    "Shared"
    "TunnelObfuscationTests"
)
cd "$script_dir"

if [[ "$command" == "lint" ]]; then
    swift format lint -r -p "$@" "${included_folders[@]}"
elif [[ "$command" == "format" ]]; then
    swift format format -r -p -i "$@" "${included_folders[@]}"
fi
