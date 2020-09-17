FROM rust:alpine AS builder

RUN apk --no-cache add g++

WORKDIR /home/rust/
COPY . .
RUN cargo test
RUN cargo build --release
RUN strip target/release/lisa

FROM alpine:latest
WORKDIR /home/lisa
COPY --from=builder /home/rust/target/release/lisa .
ENV PATH="${PATH}:/home/lisa"
