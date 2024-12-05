FROM rustlang/rust:nightly-bookworm

WORKDIR /
COPY ../ .

ENV SQLX_OFFLINE=true
RUN cargo install --path ./raildepot

CMD [ "raildepot" ]
