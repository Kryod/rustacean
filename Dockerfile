FROM rust:1.30.1-slim

RUN apt-get update -y

RUN apt-get upgrade -y

RUN apt-get install libssl-dev -y

RUN apt-get install pkg-config -y

RUN apt-get install nodejs -y

RUN apt-get install php -y

COPY ./ /home

WORKDIR /home

RUN cargo build

CMD ["cargo", "run"]