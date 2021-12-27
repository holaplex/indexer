FROM rust:latest
RUN apt update && apt install
RUN apt-get install libudev-dev

ARG PORT
ENV PORT=$PORT

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

RUN cargo install diesel_cli --no-default-features --features postgres

COPY ./ ./

RUN cargo build --release
