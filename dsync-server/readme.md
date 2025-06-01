# Db setup

1. First we need `diesel_cli` installation

  See [here](https://diesel.rs/guides/getting-started#installing-diesel-cli).
  tldr: run `cargo install diesel_cli --no-default-features --features sqlite`.

2. Create the database & apply migrations

  Run `diesel database setup`

  If this fails for reason related to `migrations` directory not existing - specify the directory
  directly by using `--migration-dir` option:

  `diesel database setup --migration-dir ./migrations`
