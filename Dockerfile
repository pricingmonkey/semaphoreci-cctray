FROM lukemathwalker/cargo-chef:0.1.71-rust-1.85.1-alpine3.20 AS chef
RUN apk --update add openssl-dev openssl-libs-static musl-dev pkgconfig
WORKDIR /usr/src/semaphoreci-cctray

FROM chef AS planner
COPY src src
COPY Cargo.lock Cargo.toml ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/semaphoreci-cctray/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY src src
COPY Cargo.lock Cargo.toml ./
RUN cargo build --release
RUN cargo test --release

FROM alpine:3.20 AS runtime
COPY --from=builder /usr/src/semaphoreci-cctray/target/release/semaphoreci-cctray /usr/local/bin/semaphoreci-cctray

#HEALTHCHECK --start-period=1m CMD curl -f http://localhost:5001/ready || exit 1

CMD [ "semaphoreci-cctray" ]