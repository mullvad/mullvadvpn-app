#!/usr/bin/env nix-shell
#! nix-shell -i sh
#! nix-shell -p go
#! nix-shell -I nixpkgs=https://github.com/NixOS/nixpkgs/archive/8c50a710ddca43d7a530fb805ad55bde8d0141c5.tar.gz

set -eu
set -o pipefail

echo "Creating device, requires sudo"
go mod tidy
sudo -k go run main.go 
