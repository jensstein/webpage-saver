#!/usr/bin/env bash

set -ex

cd $(dirname "$0")/../backend

NAME=woom
PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("", 0)); print(s.getsockname()[1]); s.close()')

POD_ID=$(podman create --replace --rm --name $NAME-test -e POSTGRES_DB=$NAME-test -e POSTGRES_PASSWORD=$NAME-test -e POSTGRES_USER=$NAME-test -p $PORT:5432 postgres:14-alpine)
podman start $POD_ID

trap "podman stop $POD_ID" EXIT

TEST_DB=postgresql://$NAME-test:$NAME-test@localhost:$PORT cargo test $@ || true
