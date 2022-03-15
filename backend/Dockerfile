FROM debian:9-slim
RUN apt update && apt install -y curl gcc gcc-arm-linux-gnueabihf pkg-config
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  -s -- -y
RUN . $HOME/.cargo/env
RUN /root/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf
RUN mkdir -p /code
CMD cp -r /readonly /code && cd /code/readonly && /root/.cargo/bin/cargo build --release
