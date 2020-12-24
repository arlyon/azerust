# azerust

Azerust is an experimental WoW server emulator for patch 3.3.5
written in Rust. Currently, it only implements the auth server.
Our goals are:

- fast
- readable
- safe
- concise

## Getting Started

We statically check our queries against the database to ensure
type safety across the board. This means we depend on access to
certain information from our database. Obviously spinning up a
db every time you build is cumbersome, so there are two options:

`.env`

```bash
DATABASE_URL=mysql://localhost/auth
SQLX_OFFLINE=true
```

The first dynamically updates queries as we go, while the second
uses the data in `sqlx-data.json`. This should always be up to date.

### Building

This part is easy. Provide the `--release` flag if you want to
make it go fast.

```
cargo run --bin auth -- help
```
