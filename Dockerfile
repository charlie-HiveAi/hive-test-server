FROM rust:1.66-buster

RUN apt-get update     && \
    apt-get upgrade -y && \
    apt-get install -y --fix-missing \
  autoconf \
  build-essential \
  cmake \
  curl \
  git \
  libtool \
  pkg-config

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

# create a new empty shell project
RUN USER=root cargo new --bin hive-test-server
WORKDIR /usr/src/app/hive-test-server

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
# ENV RUST_BACKTRACE 1
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/hive*
RUN cargo build --release
