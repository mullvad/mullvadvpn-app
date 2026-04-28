
# Android team build machine setup

Currently, the Android team's self hosted build infrastructure consists of our build server (named **bender**) and our
router-as-a-service (RAAS) (named **terminator**).

**bender** is our CI build server that builds, lints and tests the app when a PR is created on GitHub. This is done by
employing a number of GitHub-runner services that each installs and runs the app on a physical Android device connected
to **bender**.

**terminator** hosts the Android team's instance of RAAS, which is used to block certain types of network traffic needed
when running our E2E tests.

Both **bender** and **terminator** are running NixOS and the complete configs are checked in to this repo.

## Configuring and deploying bender

**bender** is configured via the `configuration.nix` file in this directory, and does not currently use Nix flakes.

To deploy a new configuration do the following:

1. Make sure you can ssh to `bender` by setting the following in `~/.ssh/config`
```
Host android-runner-bender
  HostName bender.local
  User runner-admin
  IdentitiesOnly yes
```
2. Get a shell with the `nixos-rebuild` command (if not already available): `nix-shell -p nixos-rebuild`
3. From this directory, run: `nixos-rebuild -I nixos-config=$(pwd)/configuration.nix --build-host android-runner-bender --target-host android-runner-bender --sudo --ask-sudo-password --no-flake switch`

## Configuring and deploying terminator

**terminator** is configured as a Nix flake called `app-team-android-lab` in the [mullvadvpn-app/ci/ios/test-router/flake.nix](../ios/test-router/flake.nix)

See the [RAAS readme](../ios/test-router/README.md) for instructions on how to deploy a new config.
