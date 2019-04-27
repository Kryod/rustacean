[![Build Status](https://travis-ci.org/Kryod/rustacean.svg?branch=master)](https://travis-ci.org/Kryod/rustacean) [![rust 1.33+ badge]][rust 1.33+ link] [![guild-badge][]][guild]



[![Discord Bots](https://discordbots.org/api/widget/509051376655925258.svg)](https://discordbots.org/bot/509051376655925258)

# Rustacean

Rustacean is a Discord bot that allows users to execute code snippets.

Made with Rust 🦀❤

## Commands

**`~help`**: Get a list of commands. Use `~help <command>` to get details on a command.

**`~languages`**: Get a list of available programming languages.

**`~stats`**: Get a list of how many snippets of a language has been executed.

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

###How does it works

The bot is supposed to run on a container (recommended), it will save all code it receive with the `exec` command in a folder `snippets/{userid}`.
Then the bot will spawn a container with an image corresponding to the language tha tis supposed to be executed, for example, when executing C code it will launch a container with the rustacean-c image (specially build using an image for that language) and then exec the code.
For security purposes, containers will have their ability restrained (not the main container), they will have no connection to the internet, they will have limited ressources (ram and cpu usage defined in the config file) and after 10sec the container will be killed and deleted (to prevent infinite loop and save space on hard drive).
For each language there is a corresponding docker image that has all the necessary dependancies(makes us able to add specific dependancies without granting access to the internet).
When launching the bot all the images will be build, so you must have enough space (the bot will automatically prune all unnecessary images after it build everything).


### Installing

⚠️ It is recommended to **[run Rustacean as a Docker container](readme/docker.md)** instead of directly on your machine, otherwise **users will be able to access and alter your file system**.

Docker is **required** for running the bot (it will spawn container). So you need to install docker.

For Linux:
```sh
user@machine:~$ apt install -y docker.io
```
For Mac:
```
user@machine:~$ brew install docker
```

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

### Adding a language

You can do a issue and hope we will have the time to do it, do a pull request or add it yourself on your fork.

In the two later choices you need to do this:

First you must make a Docker image in the folder `images` following this synthax `Dockerfile.{language}`.
If code runs on your image then the bot will be able to use it.

Then you need to make a rust file in `src/commands/exec` named `{language}.rs`.
You can copy another file from the same folder, the architecture is pretty much the same.
Take a look at `language.rs`, it will be the trait that is implemented by your language struct.
In the `get_image_name()` function you must put the name of the image that will be build, must be like `rustacean-{language}`.
Everything else is self-explanatory, if your language is interpreted then you need to tell what the interpreter is, if you have a special command for the execution you have to specify it in `get_execution_command()`...
One thing to note is that you must have the exact name between the image name after the `-` (`rustacean-{language}`), the Dockerfile extension (`Dockerfile.{language}`) and the name in the function `get_lang_name`.

After your file is done, you need to add it in `mod.rs` in the same folder by adding the following lines:
```rust
mod {language};
pub use self::{language}::{language};
```

Now you need to add the names used in the `~exec` command that will invoke the language.
This happen in `src/lang_manager.rs`, add the following line:
```rust
 mngr.languages.insert(vec![
    "invocation_name_1".into(),
    "invocation_name_2".into()
    ], Arc::new(Box::new({language})));
```

And now you are done. But if you want your pull request to be merged you need to add a test for your language, it is in the `src/test.rs` file.
After your PR is validated by travis we will happilly merge it.

[guild]: https://discord.gg/2qjtv2H
[guild-badge]: https://img.shields.io/discord/509055716305141780.svg?style=flat-square&colorB=7289DA
[rust 1.33+ badge]: https://img.shields.io/badge/rust-1.33+-93450a.svg?style=flat-square
[rust 1.33+ link]: https://blog.rust-lang.org/2019/02/28/Rust-1.33.0.html