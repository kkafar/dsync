FROM rust:1.91-alpine AS builder
WORKDIR /dsync
COPY dsync-cli/ ./dsync-cli/
COPY dsync-proto/ ./dsync-proto/
COPY dsync-server/ ./dsync-server/
COPY dsync-shared/ ./dsync-shared/
COPY config/ ./config/
COPY Cargo.lock Cargo.toml ./

FROM builder AS proto-build
RUN apk update && apk add protoc
RUN cargo build --release -p dsync-proto

FROM proto-build AS server-build
RUN apk update && apk add sqlite-dev sqlite sqlite-static
RUN cargo build --release -p dsync-server
RUN cargo install diesel_cli --no-default-features --features sqlite
RUN cd ./dsync-server && diesel database setup --migration-dir ./migrations --config-file ./diesel.toml --database-url "/dsync/main.db" && cd ../
EXPOSE 50051
# CMD ["/bin/sh"]
CMD ["cargo", "run", "--release", "-p", "dsync-server", "--", "--env-file", "./config/.test.env", "-l", "trace"]

FROM proto-build AS cli-build
RUN cargo build --release -p dsync-cli



