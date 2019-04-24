FROM rust:1.33.0-slim as builder

RUN apt-get update -y && apt-get install -y libssl-dev \
    pkg-config \
    libsqlite3-dev \
    curl

RUN curl -sSL https://get.docker.com/ | sh

RUN cargo install diesel_cli --no-default-features --features "sqlite"

COPY ./ /home

WORKDIR /home

ENV DOCKER_ENV=true

RUN cargo build --release


FROM scratch:latest

WORKDIR /home

RUN mkdir snippets

COPY --from=builder /home/target/release/rustacean /home
COPY --from=builder /home/rustacean.sqlite3 /home
COPY --from=builder /home/config.toml /home

CMD ["rustacean"]
