{
  lib,
  stdenv,
  rustPlatform,
  pkg-config,
  libmnl,
  libnftnl,
  libpcap,
  ...
}:

let
  manifest = (lib.importTOML ./raas/Cargo.toml).package;
in rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  src = ./raas;
  cargoLock.lockFile = ./raas/Cargo.lock;

  nativeBuildInputs = [ pkg-config ];
  buildInputs =
    [ libpcap ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [
      libmnl
      libnftnl
    ];
}
