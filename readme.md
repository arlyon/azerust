# azerust

Azerust is an experimental WoW server emulator for patch 3.3.5
written in Rust. Currently, it only implements the auth server.
It is currently being built to be compatible (read: piggy back
on top of) the TrinityCore database. Note, however, that we use
mariadb in the docker-compose file. Our goals are:

- fast
- readable
- safe
- concise

> **Note:** For the time being this relies on nightly for the
> `arbitrary_enum_discriminant` feature, which simplifies
> serialization / deserialization.

## Getting Started

This project uses [`cargo-make`](https://github.com/sagiegurari/cargo-make)
for scripting / tasks.

### Configuration

You will need to provide some configs. Run the `init` command on
the world and auth server to generate configs for them.

### Docker

The simplest method to kickstart is to just use docker compose.
To set up the database, you can use a docker compose file we have
available. We need to populate a schema, for which we piggy back
off the incredible Trinitycore project.

```bash
> cargo make fetch-db
> docker compose up
```

The downloaded SQL scripts will be used to set up the databases.
You will also need to create a `config-auth-compose.yml` and a
`confit-world-compose.yml` file which are used by docker-compose.

### Building

We statically check our queries against the database to ensure
type safety across the board. This means we depend on access to
certain information from our database. Obviously spinning up a
db every time you build is cumbersome, so there are two options:

`.env`

```bash
DATABASE_URL=mysql://localhost/auth
# or
SQLX_OFFLINE=true
```

Setting `DATABASE_URL` dynamically updates queries as we go, while
the `SQLX_OFFLINE` option uses the data in `sqlx-data.json`. This
should always be up to date. If the queries change at any time, we
will need to regenerate this file from the live database. To do
this, use the make command:

```bash
cargo make prepare
```

The next part is easy. Provide the `--release` flag if you want to
make it go fast.

```bash
cargo make auth
# or
cargo run --bin azerust-auth --release
```

### Logging

You can use the [RUST_LOG env var](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html) to configure log levels.
For example, we can enable debug mode for the azerust packages:

```bash
RUST_LOG=azerust_auth,azerust_world=debug
```

## Account Creation

To create a command, you can use the `exec` command to run commands
against the database.

```bash
cargo make auth exec account create <username> <password> <email>
```
