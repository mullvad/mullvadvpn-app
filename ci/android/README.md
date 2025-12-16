# How to build from local machine and deploy to build server
1. Get a shell with the `nixos-rebuild` command (if not already available): `nix-shell -p nixos-rebuild`
2. Run: `nixos-rebuild -I nixos-config=$(pwd)/configuration.nix --build-host android-github-runner --target-host android-github-runner --sudo --ask-sudo-password --no-flake switch`

