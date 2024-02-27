FROM rust:1.76 as build

WORKDIR /app

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# create dummy main file
RUN mkdir src && echo "fn main() { println!(\"build is broken\") }" > ./src/main.rs

# this build step will cache your dependencies
RUN cargo build --release

# copy your source tree
COPY ./src ./src

# the last modified attribute of main.rs needs to be updated manually,
# otherwise cargo won't rebuild it.
RUN touch -a -m ./src/main.rs

# build for release
RUN cargo build --release

# our final base
FROM debian:12-slim

ENV ML_PROVIDER cpu
ENV ORT_DYLIB_PATH /onnxruntime-linux-x64-1.17.0/lib/libonnxruntime.so

# copy the build artifact from the build stage
COPY --from=build /app/target/release/ml-backend .

RUN apt-get update && apt-get install -y wget \
    && wget https://github.com/microsoft/onnxruntime/releases/download/v1.17.0/onnxruntime-linux-x64-1.17.0.tgz \
    && tar -xvzf onnxruntime-linux-x64-1.17.0.tgz \
    && rm -f onnxruntime-linux-x64-1.17.0.tgz

# set the startup command to run your binary
CMD ["./ml-backend"]
