#! /bin/bash
mkdir -p ../../android/app/src/main/jniLibs
mkdir -p ../../android/app/src/main/jniLibs/x86
mkdir -p ../../android/app/src/main/jniLibs/arm64-v8a
mkdir -p ../../android/app/src/main/jniLibs/armeabi-v7a
cp ./target/i686-linux-android/release/libmullvad.so ../../android/app/src/main/jniLibs/x86/libmullvad.so
cp ./target/aarch64-linux-android/release/libmullvad.so ../../android/app/src/main/jniLibs/arm64-v8a/libmullvad.so
cp ./target/armv7-linux-androideabi/release/libmullvad.so ../../android/app/src/main/jniLibs/armeabi-v7a/libmullvad.so
