# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

# 1. OS (Alpine 3.19): We start with 'rust:alpine3.19' to ensure the Build OS matches the 
#    Runtime OS (Alpine 3.19) exactly.
# 2. Compiler (Nightly): We manually install Nightly Rust because some dependencies 
#    are pulling bleeding-edge versions that require Edition 2024.
FROM rust:alpine3.19 AS build
WORKDIR /usr/src/app
COPY . .

# Install build dependencies
# Install openssl-dev to match Alpine 3.19's default OpenSSL version.
# This avoids runtime errors such as "libssl.so.55 not found".
RUN apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc openssl-dev git

# Install and switch to Nightly Rust
# The code dependencies (specifically base64ct/home) require Edition 2024/Rust Nightly
RUN rustup toolchain install nightly && rustup default nightly

# Set RUSTFLAGS to disable static C runtime linkage and ensure dynamic linking with musl for Alpine compatibility.
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN cd abigen  \
    && cargo run -- --contract AssetFactory --abi-source "../smart-contracts/AssetFactory.json" \
    && cargo run -- --contract Asset --abi-source "../smart-contracts/Asset.json" \
    && cd ..

RUN cargo install --path ./actix-server

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:3.19

# Install runtime dependencies matching the build stage's alpine version
# Install openssl to ensure compatibility with the build stage
RUN apk add --no-cache libgcc libstdc++ openssl

COPY --from=build /usr/local/cargo/bin/actix-trust-service /usr/local/bin/actix-trust-service
COPY --from=build /usr/local/cargo/bin/gen-mnemonic /usr/local/bin/gen-mnemonic
COPY --from=build /usr/local/cargo/bin/deploy-contracts /usr/local/bin/deploy-contracts

# Credentials and network settings come from the runtime environment (see README).
# The wallet, the DID key storage and the access log live here; mount a volume to keep them.
WORKDIR /var/lib/trust

# LOG_FILE_NAME doubles as the MongoDB key for the log's CID, so it stays a bare file name.
# WORKDIR puts the file in the volume.
ENV RUST_LOG=info \
    RUNNING_IN_DOCKER=true \
    ADDR_D=0.0.0.0 \
    PORT=8081 \
    WALLET_DB_PATH=/var/lib/trust/walletdb \
    STRONGHOLD_SNAPSHOT_PATH=/var/lib/trust/snapshot.stronghold \
    KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH=/var/lib/trust/key_storage.stronghold \
    LOG_FILE_NAME=dlog.log

EXPOSE 8081
ENTRYPOINT [ "actix-trust-service" ]
