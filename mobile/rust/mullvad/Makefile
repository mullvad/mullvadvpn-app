ARCHS_ANDROID = aarch64-linux-android armv7-linux-androideabi i686-linux-android
LIB=libsigner.a
all: android
android: $(ARCHS_ANDROID)
	sh copy_android.sh
.PHONY: $(ARCHS_ANDROID)
$(ARCHS_ANDROID): %:
	cargo build --target $@ --release