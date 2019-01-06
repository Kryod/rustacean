FROM kryod/rustacean-test:lastest

RUN cargo build --release

CMD ["cargo", "run", "--release"]
