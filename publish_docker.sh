#!/bin/bash

echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin
docker push kryod/rustacean-test
docker build -t kryod/rustacean -f Dockerfile .
docker push kryod/rustacean