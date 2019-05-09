## Linux prerequisites

```bash
$ sudo apt-get update -y
$ sudo apt-get install -y gcc curl libssl-dev libsqlite3-dev docker.io
$ curl https://sh.rustup.rs -sSf | sh
```
Press "1" and then hit Enter.

Wait for the installation to end, and then run
```bash
$ source $HOME/.cargo/env
$ cargo install diesel_cli --no-default-features --features "sqlite"
```