# Running Rustacean as a Docker container


## Contribute

To work on the bot with docker you must build the Dockerfile.test image. Don't forget to set your config.toml

```sh
user@machine:~$ sudo docker build -t rusttest -f Dockerfile.test .
```

Now you have an environnement with all the required dependancies. To start working you need to run the docker and link it with the folder where you "git cloned" this repository.

```sh
user@machine:~$ sudo docker run -it -v /path/to/rustacean:/home rusttest
```

Now your changes locally will affect files on the docker and vice-versa. You can launch the bot with `cargo run` or test with `cargo test`. 

## Deploy

To deploy the bot you need to make the image. Do not forget to set your config.toml.
```sh
user@machine:~/rustacean$ sudo docker build -t rustacean .
```

Now you want this image to run on a server or something that will be on 24/7. You will probably want to have access to the logs of the bot if it crashes so you need to link the `rustacean.log` file in this directory to the `rustacean.log` file on the docker container.
```sh
user@machine:~/rustacean$ touch rustacean.log
user@machine:~/rustacean$ sudo docker run -t --restart="always" -d -v "$(pwd)/rustacean.log":/home/rustacean.log rustacean
user@machine:~/rustacean$ tail -f rustacean.log # you can now run this to monitor the bot
```
