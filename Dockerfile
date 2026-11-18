# Build stage
FROM rust:slim-trixie AS build-stage

## Upgrade system packages
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends gcc git make mold

## Configure the build environment and working directory
WORKDIR /app

## Pre-install toolchain using rust-toolchain.toml
COPY ./rust-toolchain.toml ./
RUN rustup show

## Copy Cargo configuration and dependency manifests for caching
COPY ./.cargo ./.cargo
COPY ./Cargo.lock ./Cargo.toml ./

## Prebuild a dummy project to populate the compiled dependency cache
## Release-mode dependencies are reused by the subsequent lint and build steps
RUN --mount=type=cache,id=ws-io-server-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=ws-io-server-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-io-server-target,target=/cargo-target \
    mkdir ./src && \
    echo 'fn main() {}' >./src/main.rs && \
    CARGO_TARGET_DIR=/cargo-target cargo b -r --all-features --all-targets --locked && \
    rm -rf ./src

## Copy application sources and build configuration
COPY ./rustfmt.toml ./
COPY ./src ./src

## Validate and build the application
RUN --mount=type=cache,id=ws-io-server-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=ws-io-server-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-io-server-target,target=/cargo-target \
    touch ./src/main.rs && \
    cargo format && \
    CARGO_TARGET_DIR=/cargo-target cargo clippy -r --all-features --all-targets --locked -- -D warnings && \
    CARGO_TARGET_DIR=/cargo-target cargo b -r --all-features --locked && \
    cp -p /cargo-target/release/ws-io-server ./ws-io-server

# Runtime stage
FROM debian:trixie-slim

## Configure the runtime environment and working directory
ENV TZ='Asia/Taipei'
WORKDIR /app

## Install runtime packages and configure the runtime user
RUN \
    ### Upgrade system packages and install runtime dependencies
    apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends ca-certificates tini tzdata && \
    ### Configure the timezone
    ln -snf "/usr/share/zoneinfo/${TZ}" /etc/localtime && \
    echo "${TZ}" >/etc/timezone && \
    ### Clean package manager metadata
    apt-get autoremove -y --purge && \
    apt-get clean && \
    rm -rf /var/cache/apt/* /var/lib/apt/lists/* && \
    ### Create the runtime user and set application ownership
    useradd -mr -g nogroup -s /usr/sbin/nologin -u 10001 user && \
    chown 10001:nogroup /app -R

## Copy and configure the entrypoint
COPY --chmod=700 --chown=10001:nogroup ./docker-entrypoint.sh ./
USER 10001
ENTRYPOINT ["tini", "--"]
CMD ["./docker-entrypoint.sh"]

## Configure remaining runtime defaults
ENV SERVER_HOST='0.0.0.0' \
    SERVER_PORT=8080

## Copy application output and runtime configuration
COPY --chown=10001:nogroup --from=build-stage /app/ws-io-server ./
COPY --chown=10001:nogroup ./.env.production.local ./
