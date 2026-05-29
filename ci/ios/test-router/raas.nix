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

rustPlatform.buildRustPackage rec {
  pname = "raas";
  version = "0.0.1";

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
