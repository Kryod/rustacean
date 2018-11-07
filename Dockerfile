FROM rust

COPY ./ /home

WORKDIR /home

RUN cargo build

CMD ["cargo", "run"]