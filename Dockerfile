FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /
COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

ENV SQLX_OFFLINE=true
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine
COPY --from=builder /target/x86_64-unknown-linux-musl/release/aurora_api ./
CMD [ "./aurora_api" ]
LABEL service=aurora-api
