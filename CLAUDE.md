# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

`dsync` is a local-network, peer-to-peer file transfer / synchronization daemon (loose subset of `rsync` semantics, with built-in host discovery). It is a Cargo workspace of four crates communicating over gRPC (tonic/prost). Each host runs a `dsync-server` daemon and is operated locally by a `dsync-cli` client; peers discover each other via `nmap` + a `HelloThere` exchange and then transfer files directly.

See `docs/local.project-desc.md` and `docs/local.system-design.md` for the design intent. The TLD design is *no* master, *no* encryption, *no* auth — it assumes a trusted LAN.

## Build, run, test

Toolchain: Rust edition 2024 (uses `cargo` workspace, `resolver = "3"`). The CI build (`.github/workflows/test-build.yaml`) requires `protoc` to be installed — `dsync-proto/build.rs` invokes `tonic-build` against every `*service.proto` under `dsync-proto/proto/` and writes generated code into `dsync-proto/proto-generated/` (which is checked in; rebuilds overwrite it).

```bash
cargo build                                  # whole workspace
cargo build --bin dsync-server               # server only (matches CI)
cargo build --bin dsync-cli                  # CLI only (matches CI)

# Run the server in dev (loads dsync-server/.env -> DATABASE_URL=./main.db, SERVER_PORT=50051)
cargo run --package dsync-server -- --log-level trace --env-file ./config/.test-env

# Run the CLI against the local server
cargo run --bin dsync-cli -- host list
cargo run --bin dsync-cli -- file copy /abs/path remote-name@/abs/dst/path
cargo run --bin dsync-cli -- server shutdown
```

### Database / migrations

SQLite via Diesel. Migrations live in `dsync-server/migrations/` and are **embedded into the binary** (`embed_migrations!()` in `src/server/data/source/sqlite/database/initialization.rs`) — they run automatically on startup against `DATABASE_URL`. `dsync-server/build.rs` reruns the build when migration files change.

`diesel.toml` points `print_schema` at `src/server/data/source/sqlite/database/schema.rs`. To add a migration you need `diesel_cli` (`cargo install diesel_cli --no-default-features --features sqlite`) and then `diesel migration generate <name>` from inside `dsync-server/`. After applying, regenerate `schema.rs` (`diesel print-schema > src/server/data/source/sqlite/database/schema.rs`) — the running daemon does not regenerate the Rust schema, only the SQL.

## Workspace layout

- **`dsync-proto`** — `.proto` files + generated Rust. Service definitions under `proto/services/{file-transfer,host-discovery,user-agent,server-control}/`, shared models under `proto/model/`. Re-exported via `dsync_proto::services::*` and `dsync_proto::model::*`. The build script picks up *only* files ending in `service.proto`, so a new RPC service must have its top-level proto named `service.proto` to be compiled.
- **`dsync-shared`** — code shared by both binaries: gRPC channel helpers (`conn.rs` — `ChannelFactory`, `create_server_uri`, `ServiceConnFactory`), the `[HOST@]PATH` parsing/wrapper types (`model.rs` — `FileSourceWrapper`, `HostSpecWrapper`, `PathSpecWrapper`), and the default port constant.
- **`dsync-cli`** — `clap` subcommand tree (`host`, `file`, `group`, `server`). Subcommand definitions live in `src/cli/command/*.rs`; their handlers (which actually make gRPC calls) live in the parallel `src/command/*.rs`. CLI talks **only** to the local `UserAgentService` (or `ServerControlService`); it never contacts remote daemons directly.
- **`dsync-server`** — the daemon. Library code in `src/server/`, binary entrypoint in `src/bin/dsync-server/`.

## Server architecture (key code patterns)

The daemon (`dsync-server/src/server.rs`) wires four tonic services onto a single port:

| Service | Source | Role |
|---|---|---|
| `UserAgentService` | `service/user_agent.rs` | Entry point for the local CLI. All user-facing commands enter here; this service is what orchestrates calls to remote peers. |
| `HostDiscoveryService` | `service/host_discovery.rs` | Handles the `HelloThere` handshake between peers. Called by other servers, not by CLIs. |
| `FileTransferService` | `service/file_transfer.rs` | Two-step flow: `TransferSubmit` (origin→source) → `TransferInit` (source→dest) → streamed `TransferChunk` (source→dest). Session state held in `session_registry`. |
| `ServerControlService` | `service/server_control.rs` | Owns the shutdown `oneshot::Sender`; receiving `shutdown` triggers `serve_with_shutdown` to return. |

**`ServerContext`** (`server/context.rs`) is the shared `Arc`-wrapped state — config + a `dyn DataRepository`. Every service receives it in its constructor.

**Data layer** is two layers of trait indirection: `DataRepository` (`server/data/repo.rs`) → `DataSource` (`server/data/source.rs`) → `SqliteDataSource` (`server/data/source/sqlite.rs`). The repository currently just forwards every call; the split exists so the data source can be swapped in tests / for other backends. Both traits must be updated together when adding a new operation. `SqliteDataSource` wraps a single `SqliteConnection` in a `tokio::sync::Mutex` (diesel's sqlite backend is sync, so all DB work serializes through this mutex — keep ops short).

**Local-server identity** is bootstrapped in `SqliteDataSource::new`: if no `is_remote = false` row exists in `hosts`, one is inserted using the closure passed in (which calls `hostname` and generates a UUID). Per-server UUIDs are stable for the life of a database; deleting `main.db` re-identifies the host to its peers.

**Cross-daemon RPC** is built ad hoc by services via `ChannelFactory::channel_with_timeout` + `create_server_uri(SocketAddrV4)`.

**Config loading** (`bin/dsync-server/config.rs`) uses a `PartialConfigProvider` chain: CLI args → env (via `dotenvy`) → XDG state dir (`xdg::BaseDirectories::with_prefix("dsync")` → `…/state/dsync/main.db`). Providers earlier in the list win on merge. To make a config field optional-then-required, add it to `PartialConfig`, implement `.merge`, and validate in `TryInto<Config>`.

## Conventions

- **Logging**: `log` + `log4rs`.
- **gRPC errors**: services use short kebab-case status messages (`"src-path-not-absolute"`, `"fts-rejected"`, `"missing-host-spec"`). Match the style when adding new error returns.
- **Path semantics**: every file path crossing the gRPC boundary is expected to be **absolute**. The CLI helper `PathSpecWrapper::try_into_abs_path` converts relative → absolute before sending; services re-check (`PathBuf::is_absolute`) and reject otherwise.
