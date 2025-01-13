
FROM rust:1.82.0 AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo 'fn main() { println!("Hello, world!"); }' > src/main.rs

RUN cargo fetch

RUN rm src/main.rs

COPY src ./src

COPY sql ./sql

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser

WORKDIR /usr/local/bin

RUN mkdir uploads && chown appuser:appuser uploads

COPY --from=builder /usr/src/app/target/release/knowledge-base .

COPY --from=builder /usr/src/app/sql ./sql

RUN chown appuser:appuser ./knowledge-base

USER appuser

EXPOSE 8000

CMD ["./knowledge-base"]
