FROM ubuntu as builder

RUN apt-get update -y
RUN apt-get install -y \
    build-essential \
    git \
    mlton \                      
    unzip

COPY rootfs/ /

WORKDIR /usr/src
ENV PATH=$PATH:/root/.smackage/bin
ENV SMACKAGE_HOME=/root/.smackage

RUN git clone https://github.com/standardml/smackage.git && cd smackage &&\
    make mlton &&\
    bin/smackage refresh &&\
    bin/smackage make smackage mlton &&\
    bin/smackage make smackage install

RUN \
    smackage get unicode &&\
    smackage get aplparse v1 &&\
    smackage get MoA v1 &&\
    git clone https://github.com/melsman/aplcompile.git && cd aplcompile &&\
    mlton -output aplc aplc.mlb

FROM debian:buster

COPY --from=builder /usr/src/aplcompile/aplc /usr/bin
#ENTRYPOINT aplc

