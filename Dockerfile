FROM rust:latest
RUN apt update && apt install
RUN apt-get install libudev-dev

ARG DATABASE_URL
ARG PORT
ENV PORT=$PORT
ENV DATABASE_URL=$DATABASE_URL

COPY ./ ./

RUN cargo build --release
