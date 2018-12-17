# Running Rustacean as a Docker container


## Contribute

To work on the bot with docker you must build the Dockerfile.test image. Don't forget to set your config.toml

```sh
user@machine:~$ docker build -t rusttest -f Dockerfile.test .
```

Now you have an environnement with all the required dependancies. To start working you need to run the docker and link it with the folder where you "git cloned" this repository.

```sh
user@machine:~$ docker run -it -v /path/to/rustacean:/home rusttest
```

Now your changes locally will affect files on the docker and vice-versa. You can launch the bot with `cargo run` or test with `cargo test`. 

## Deploy

To deploy the bot you need to make the image. Do not forget to set your config.toml.
```sh
user@machine:~$ docker build -t rustacean .
```

Now you want this image to run on a server or something that will be on 24/7. You will probably want to have access to the logs of the bot if it crashes so you need to link the rustacean.log (which do not exist in this repository so do `touch rustacean.log`) to the rustacean.log file on the docker.
```sh
user@machine:~$ docker run -t --restart="always" -d -v /path/to/rustacean/rustacean.log:/home/rustacean.log rustacean
```