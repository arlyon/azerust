FROM clux/muslrust:nightly as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare

FROM chef as cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release

FROM chef as builder
COPY . .
COPY --from=cacher /app/target target
ENV SQLX_OFFLINE true
RUN cargo build --bin azerust-auth --release

FROM scratch as auth
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/azerust-auth /auth
CMD ["/auth", "run"]
