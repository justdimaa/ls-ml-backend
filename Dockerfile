FROM rust:1.76 as build

# create a new empty shell project
# RUN USER=root cargo new --bin ml-backend
WORKDIR /app

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
# RUN cargo build --release
# RUN rm -f src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
# RUN rm -f ./target/release/deps/ml-backend*
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
