FROM rust:1.39-stretch AS builder

WORKDIR /usr/src/bba
COPY . .

RUN rustup default nightly
RUN rustup update
RUN cargo install --path .
RUN cargo build --release

CMD ["target/release/bba"]