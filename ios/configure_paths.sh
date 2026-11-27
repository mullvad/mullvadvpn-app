#!/usr/bin/env bash
set -euo pipefail

# Prepend a directory to PATH if it exists and isn't already present.
path_prepend() {
    if [ -d "$1" ]; then
        case ":$PATH:" in
        *":$1:"*) ;; # already present
        *) export PATH="$1:$PATH" ;;
        esac
    fi
}

add_nix() {
    # Add the per-user profile (NixOS / nix-darwin)
    path_prepend "/etc/profiles/per-user/$USER/bin"

    # Add nix-profile if in use (home-manager, nix-profile install)
    path_prepend "$HOME/.nix-profile/bin"
}

add_homebrew() {
    case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
        arm64) path_prepend "/opt/homebrew/bin" ;;
        x86_64) path_prepend "/usr/local/bin" ;;
        esac
        ;;
    Linux)
        path_prepend "/home/linuxbrew/.linuxbrew/bin"
        path_prepend "$HOME/.linuxbrew/bin"
        ;;
    esac
}

add_nix
add_homebrew
