PROJECTS := colmeia-dat1 colmeia-dat1-ffi colmeia-dat1-mdns colmeia-dat1-core colmeia-dat1-proto

RUST_FILES := $(foreach dir,$(PROJECTS),$(shell find $(dir)/src -name *.rs))

MODE := debug

ifeq ($(MODE),release)
	RUST_FLAGS=--release
endif

RUST_TARGETS := aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
ANDROID_LIBS := $(foreach target,$(RUST_TARGETS),target/$(target)/$(MODE)/libcolmeia_dat1_ffi.so)

target/%/$(MODE)/libcolmeia_dat1_ffi.so: $(RUST_FILES)
	cross build -p colmeia-dat1-ffi --target $* $(RUST_FLAGS)

android: $(ANDROID_LIBS)
	-cp target/aarch64-linux-android/$(MODE)/libcolmeia_dat1_ffi.so \
		flutter/colmeia_native/android/src/main/jniLibs/arm64-v8a/libcolmeia_dat1_ffi.so

	-cp target/armv7-linux-androideabi/$(MODE)/libcolmeia_dat1_ffi.so \
		flutter/colmeia_native/android/src/main/jniLibs/armeabi-v7a/libcolmeia_dat1_ffi.so

	-cp target/x86_64-linux-android/$(MODE)/libcolmeia_dat1_ffi.so \
		flutter/colmeia_native/android/src/main/jniLibs/x86_64/libcolmeia_dat1_ffi.so

	-cp target/i686-linux-android/$(MODE)/libcolmeia_dat1_ffi.so \
		flutter/colmeia_native/android/src/main/jniLibs/x86/libcolmeia_dat1_ffi.so

target/universal/$(MODE)/libcolmeia_dat1_ffi.a: $(RUST_FILES)
	cargo lipo $(RUST_FLAGS)

ios: target/universal/$(MODE)/libcolmeia_dat1_ffi.a
	-cp $< flutter/colmeia_native/ios/libs/libcolmeia_dat1_ffi.a

.PHONY: android ios
