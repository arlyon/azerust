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

### Configuration

You will need to provide some configs. Run `azerust init` to
generate a default file.

### Docker

The simplest method to kickstart is to just use docker compose.
To set up the database, you can use a docker compose file we have
available. We need to populate a schema, for which we piggy back
off the incredible Trinitycore project.

```bash
> cd scripts
> ./setup.sh
> cd -
> docker compose up
```

You will also need to create a `config-compose.yml` which is used
by the image to  

The downloaded SQL scripts will be used to set up the databases.

### Building

We statically check our queries against the database to ensure
type safety across the board. This means we depend on access to
certain information from our database. Obviously spinning up a
db every time you build is cumbersome, so there are two options:

`.env`
```bash
DATABASE_URL=mysql://localhost/auth
SQLX_OFFLINE=true
```

Setting `DATABASE_URL` dynamically updates queries as we go, while
the `SQLX_OFFLINE` option uses the data in `sqlx-data.json`. This
should always be up to date. If the queries change at any time, we
will need to regenerate this file from the live database. To do
this, use the sqlx cli.

```bash
cargo install sqlx-cli
cargo sqlx prepare --merged
```

The next part is easy. Provide the `--release` flag if you want to
make it go fast.

```
cargo run --bin auth --release -- log
```

### Logging

You can use the [RUST_LOG env var](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html) to configure log levels.
For example, we can enable debug mode for the azerust packages:

```
RUST_LOG=auth=debug,mysql=debug,game=debug azerust log
```

## Account Creation

To create a command, you can use the `exec` command to run commands 
against the database.

```
azerust exec account create <username> <password> <email>
```