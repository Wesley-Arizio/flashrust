FROM rust:1.88-bullseye as builder
WORKDIR /app

COPY . .

RUN cargo build --release --package auth

FROM debian:bullseye-slim
WORKDIR /app

COPY auth/.env /app/.env

COPY --from=builder /app/target/release/auth /app/auth

RUN chmod +x /app/auth

ENTRYPOINT ["/app/auth"]
EXPOSE 3000
