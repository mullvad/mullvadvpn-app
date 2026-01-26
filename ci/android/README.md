# How to build from local machine and deploy to build server
1. Make sure you can ssh to `bender` by setting the following in `~/.ssh/config`
```
Host android-runner-bender
  HostName bender.local
  User runner-admin
  IdentitiesOnly yes
```
2. Get a shell with the `nixos-rebuild` command (if not already available): `nix-shell -p nixos-rebuild`
3. Run: `nixos-rebuild -I nixos-config=$(pwd)/configuration.nix --build-host android-runner-bender --target-host android-runner-bender --sudo --ask-sudo-password --no-flake switch`

