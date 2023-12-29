#!/usr/bin/env bash

FFI_TARGET=abstract-tun

RELFLAG=
if [[ "$CONFIGURATION" -eq "Release" ]]; then
  RELFLAG=--release
fi

set -euvx

# if [[ -n "${SDK_DIR:-}" ]]; then
  # Assume we're in Xcode, which means we're probably cross-compiling.
  # In this case, we need to add an extra library search path for build scripts and proc-macros,
  # which run on the host instead of the target.
  # (macOS Big Sur does not have linkable libraries in /usr/lib/.)
      # export LIBRARY_PATH="${SDK_DIR}/usr/lib:${LIBRARY_PATH:-}"
# fi

 IS_SIMULATOR=0
 if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
   IS_SIMULATOR=1
 fi

 function run_build {

     local target=$1
     env -i zsh -c \
         "CFLAGS_x86_64_apple_ios='-target x86_64-apple-ios' \
          $HOME/.cargo/bin/cargo build -p $FFI_TARGET \
            --lib $RELFLAG \
            --target $target"
 }

 echo "THE LIB PATH IS ${DEVELOPER_SDK_DIR}"
 for arch in $ARCHS; do
   case "$arch" in
     x86_64)
       if [ $IS_SIMULATOR -eq 0 ]; then
         echo "Building for x86_64, but not a simulator build. What's going on?" >&2
         exit 2
       fi

       # Intel iOS simulator
       run_build x86_64-apple-ios
       ;;

     arm64)
       if [ $IS_SIMULATOR -eq 0 ]; then
         # Hardware iOS targets
           run_build aarch64-apple-ios
       else
           run_build aarch64-apple-ios-sim
       fi
   esac
 done
