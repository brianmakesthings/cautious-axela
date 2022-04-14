.PHONY: all clean

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc

DEPLOY_PATH= $(HOME)/cmpt433/public/myApps/server-copy

all:
	mkdir -p $(DEPLOY_PATH)
	chmod a+rwx $(DEPLOY_PATH)
	cp -r ../frontend/ $(DEPLOY_PATH)

	cargo build --release --target armv7-unknown-linux-gnueabihf
	cp ./target/armv7-unknown-linux-gnueabihf/release/server $(DEPLOY_PATH)
	cp ./target/armv7-unknown-linux-gnueabihf/release/intercom $(DEPLOY_PATH)
	cp ./code $(DEPLOY_PATH)
	cp ./.env $(DEPLOY_PATH)

clean:
	cargo clean
