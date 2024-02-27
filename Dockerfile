FROM rust:alpine AS builder

RUN apk --no-cache add g++

WORKDIR /home/rust/
COPY . .
RUN cargo test
RUN cargo build --release

RUN strip target/release/lisi

FROM alpine:latest
WORKDIR /home/lisi
COPY --from=builder /home/rust/target/release/lisi .
ENV PATH="${PATH}:/home/lisi"
