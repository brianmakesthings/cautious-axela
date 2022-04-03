.PHONY: all clean

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc

DEPLOY_PATH= $(HOME)/cmpt433/public/myApps/server-copy

all:
	mkdir -p $(DEPLOY_PATH)
	chmod a+rwx $(DEPLOY_PATH)
	cp -r ../frontend/index.html $(DEPLOY_PATH)
	cp -r ../frontend/stylesheets/style.css $(DEPLOY_PATH)

	cargo build --release --target armv7-unknown-linux-musleabihf
	cp ./target/armv7-unknown-linux-musleabihf/release/server $(DEPLOY_PATH)

clean:
	cargo clean
