FROM rust:1.31 AS builder

WORKDIR /usr/src/bba
COPY . .

RUN cargo install --path .

CMD ["cargo build --release"]

FROM alpine:3.7

WORKDIR /usr/src/bba
COPY --from=builder /usr/src/bba/target/release/bba .
CMD ["./bba"]
