FROM kryod/rustacean-test:latest

RUN cargo build --release

CMD ["cargo", "run", "--release"]
