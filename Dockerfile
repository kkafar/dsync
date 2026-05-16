ARG PKG_SQLITE_VERSION=3.51.2-r0
ARG PKG_PROTOC_VERSION=31.1-r1

FROM rust:1.95-alpine3.23 AS builder
ARG PKG_SQLITE_VERSION
ARG PKG_PROTOC_VERSION
RUN apk add --no-cache \
  protoc="${PKG_PROTOC_VERSION}" \
  sqlite-dev="${PKG_SQLITE_VERSION}" \
  sqlite="${PKG_SQLITE_VERSION}" \
  sqlite-static="${PKG_SQLITE_VERSION}"
WORKDIR /dsync
COPY dsync-cli/ ./dsync-cli/
COPY dsync-proto/ ./dsync-proto/
COPY dsync-server/ ./dsync-server/
COPY dsync-shared/ ./dsync-shared/
COPY config/ ./config/
COPY Cargo.lock Cargo.toml ./

FROM builder AS dsync-build-release
RUN cargo build --release --workspace

FROM alpine:3.23.4 AS dsync-release
ARG PKG_SQLITE_VERSION
RUN apk add --no-cache sqlite="${PKG_SQLITE_VERSION}"
COPY --from=dsync-build-release /dsync/target/release/dsync-server /usr/local/bin/
COPY --from=dsync-build-release /dsync/target/release/dsync-cli /usr/local/bin/
COPY --from=dsync-build-release /dsync/config /dsync/config
EXPOSE 50051
CMD ["dsync-server", "--env-file", "/dsync/config/.test.env", "--log-level", "trace"]
