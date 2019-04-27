FROM ubuntu:latest

RUN apt-get update -y && apt-get install -y lua5.3 \
    lua-socket \
    lua-sec \