FROM rust:1.99-alpine3.24 AS builder

RUN apk --update add openssl-dev openssl-libs-static musl-dev pkgconfig

WORKDIR /usr/src/semaphoreci-cctray

COPY src src
COPY Cargo.lock Cargo.toml ./
RUN cargo build --release
RUN cargo test --release

FROM alpine:3.24 AS runtime
COPY --from=builder /usr/src/semaphoreci-cctray/target/release/semaphoreci-cctray /usr/local/bin/semaphoreci-cctray

RUN apk --no-cache upgrade zlib

#HEALTHCHECK --start-period=1m CMD curl -f http://localhost:5001/ready || exit 1

CMD [ "semaphoreci-cctray" ]