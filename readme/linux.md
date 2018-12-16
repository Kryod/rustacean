## Linux prerequisites

```bash
$ sudo apt-get update -y
$ sudo apt-get install -y gcc curl libssl-dev
$ curl https://sh.rustup.rs -sSf | sh
```
Press "1" and then hit Enter.

Wait for the installation to end, and then run
```bash
$ source $HOME/.cargo/env
```

### Languages

You can support different optional languages:

#### C++

```bash
$ sudo apt-get install -y g++
```

#### PHP

```bash
$ sudo apt-get install -y php
```

#### Python

```bash
$ sudo apt-get install -y python3
```

#### JavaScript

```bash
$ sudo apt-get install -y nodejs
```

#### C#

```bash
$ sudo apt-get install -y mono-devel
```

#### Java

```bash
$ sudo apt-get install -y default-jdk
```
