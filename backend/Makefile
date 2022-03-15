.PHONY: all clean compile build

DEPLOY_PATH= $(HOME)/cmpt433/public/myApps/server-copy

all: build compile

build:
	docker build . -t cautious-axela
	docker create --name cautious-axela -v $(PWD):/readonly/ cautious-axela

compile:
	docker start -a cautious-axela
	mkdir -p $(DEPLOY_PATH)
	chmod a+rwx $(DEPLOY_PATH)
	cp -r ../frontend/index.html $(DEPLOY_PATH)
	cp -r ../frontend/stylesheets/style.css $(DEPLOY_PATH)
	docker cp cautious-axela:/code/readonly/target/armv7-unknown-linux-gnueabihf/release/server $(DEPLOY_PATH)

clean:
	-docker stop cautious-axela
	-docker rm cautious-axela
	cargo clean
