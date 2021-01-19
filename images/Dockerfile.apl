FROM ubuntu as builder

RUN apt-get update -y
RUN apt-get install -y \
    build-essential \
    git \
    mlton \                      
    unzip

RUN mkdir -p /usr/lib/mlton/ && touch /usr/lib/mlton/mlb-path-map && mkdir -p /root/.smackage/ && touch /root/.smackage/sources.local
RUN echo "MLTON_ROOT \$(LIB_MLTON_DIR)/sml\nSML_LIB \$(LIB_MLTON_DIR)/sml\nSMACKAGE /root/.smackage/lib" >> /usr/lib/mlton/mlb-path-map
RUN echo "aplparse git git://github.com/melsman/aplparse.git\nMoA git git://github.com/melsman/MoA.git\nkitlib git git://github.com/melsman/kitlib.git\nsmackage git git://github.com/standardml/smackage.git\naplc git git://github.com/melsman/aplcompile.git" >> /root/.smackage/sources.local

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