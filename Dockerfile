FROM rust:1-slim-bullseye AS builder
WORKDIR /build

# cache the compiled dependencies
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# build the project
COPY . .
RUN cargo build --release



FROM debian:bullseye-slim AS app
WORKDIR /bot
COPY --from=builder /build/target/release/app /bot/app
CMD ["./app"]
