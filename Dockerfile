FROM rust:1.30.1-slim

RUN apt-get update -y && apt-get install -y libssl-dev \
    pkg-config \
    nodejs \
    php \
    apt-transport-https \
    dirmngr \ 
    mono-devel 

COPY ./ /home

WORKDIR /home

RUN cargo build

CMD ["cargo", "run"]
