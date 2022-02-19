.PHONY: all clean

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc

all:
	cargo build --release --target armv7-unknown-linux-musleabihf

clean:
	cargo clean
