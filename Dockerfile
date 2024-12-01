FROM rustlang/rust:nightly-alpine

WORKDIR /
COPY . .

RUN cargo install --path .

CMD ["aurora_api"]
