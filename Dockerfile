FROM rust:1-slim-bullseye AS builder
WORKDIR /build

# build the project
COPY . .
RUN cargo build --release



FROM debian:bullseye-slim AS app
WORKDIR /bot
COPY --from=builder /build/target/release/app /bot/app
CMD ["./app"]
