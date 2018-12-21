[![Build Status](https://travis-ci.org/Kryod/rustacean.svg?branch=master)](https://travis-ci.org/Kryod/rustacean)

# Rustacean

Rustacean is a Discord bot that allows users to execute code snippets.

Made with Rust 🦀❤

## Commands

**`~help`**: Get a list of commands. Use `~help <command>` to get details on a command.

**`~languages`**: Get a list of available programming languages.

**`~git`**: Get a link to the bot's GitHub repository.

**`~exec`**: Executes a code snippet. Your message needs to look like this:
````
~exec
​```language

code...
​```
````
where `language` is the language of your choice.
For example:
````
~exec
​```c
printf("Oh hi Discord!");
​```
````

## Getting Started

### Prerequisites

[prerequisites on Windows](readme/windows.md)

[prerequisites on Linux](readme/linux.md)

[prerequisites on macOS](readme/macos.md)

### Installing

⚠️ It is recommended to **[run Rustacean as a Docker container](readme/docker.md)** instead of directly on your machine, otherwise **users will be able to access your file system**.

Create a clone of this project on your development machine:
```sh
user@machine:~$ git clone https://github.com/Kryod/rustacean.git
```

Register a Discord bot here https://discordapp.com/developers/applications/me

Set up your environment:
```sh
user@machine:~$ cd rustacean/ # Go to your copy of this repository
user@machine:~/rustacean$ cargo run update-db
user@machine:~/rustacean$ cp config.toml.example config.toml
user@machine:~/rustacean$ nano config.toml # Edit this file to set your Discord bot credentials
```

Then, you can run the bot:
```sh
user@machine:~/rustacean$ cargo run
```
When the program starts running, an invite link will be printed out to add the bot to your Discord server.

### Docker

You can also [run Rustacean as a Docker container](readme/docker.md).
