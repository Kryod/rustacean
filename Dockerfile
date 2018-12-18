FROM rust:1.30.1-slim

RUN mkdir -p /usr/share/man/man1

RUN apt-get update -y && apt-get install -y libssl-dev \
    pkg-config \
    python3 \
    g++ \
    nodejs \
    php-cli \
    mono-devel \
    mono-vbnc \
    lua5.3 \
    lua-socket \
    lua-sec \
    openjdk-8-jdk \
    nasm

RUN apt-get install -y wget unzip && \
    cd /usr/lib && \
    wget https://github.com/JetBrains/kotlin/releases/download/v1.3.20-eap-25/kotlin-compiler-1.3.20-eap-25.zip && \
    unzip kotlin-compiler-*.zip && \
    rm kotlin-compiler-*.zip && \
    rm -f kotlinc/bin/*.bat && \
    apt-get remove -y wget unzip

ENV PATH $PATH:/usr/lib/kotlinc/bin

COPY ./ /home

WORKDIR /home

RUN cargo build

CMD ["cargo", "run"]
