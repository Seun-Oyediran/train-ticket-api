#stage 1
FROM rust:1.98.1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release


#stage 2
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/train-ticket .
EXPOSE 8080
CMD ["./train-ticket"]
