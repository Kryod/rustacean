FROM arm32v7/rust:1.30.1-slim

RUN apt-get update -y

RUN apt-get upgrade -y

RUN apt-get install libssl-dev -y

RUN apt-get install pkg-config -y

RUN apt-get install nodejs -y

RUN apt-get install php -y

RUN apt-get install apt-transport-https dirmngr -y
RUN apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF
RUN echo "deb https://download.mono-project.com/repo/debian stable-raspbianstretch main" | sudo tee /etc/apt/sources.list.d/mono-official-stable.list
RUN apt-get update -y
RUN apt-get install mono-devel -y

COPY ./ /home

WORKDIR /home

RUN cargo build

CMD ["cargo", "run"]