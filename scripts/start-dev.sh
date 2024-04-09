#!/usr/bin/env bash

set -xe

cd $(realpath $(dirname $0))/..

source frontend/.env.local

if test "$(podman images localhost/yarn --format {{.Digest}})" = ""; then
	pushd docker
	podman build -t yarn -f Dockerfile-node .
	popd
fi

if test "$(podman images localhost/rust-dev --format {{.Digest}})" = ""; then
	pushd docker
	podman build -t rust-dev -f Dockerfile-rust .
	popd
fi

NAME=woom
POD_ID=$(podman pod create --replace -n $NAME-dev -p 3000:3000 -p 5000:5000 -p 5432:5432 --userns keep-id)

# When userns=keep-id is set on the postgres container the id in the container
# changes to the id of the user who started it. But that's a problem a problem
# because the container needs to run as user 0 to be able to start the database.
podman create --pod $POD_ID --name $NAME-pg -e POSTGRES_DB=$NAME -e POSTGRES_PASSWORD=$NAME -e POSTGRES_USER=$NAME -u 0 -v $PWD/pg-data:/var/lib/postgresql/data:Z docker.io/postgres:14-alpine
podman create --pod $POD_ID --workdir /home/rust/source -e CARGO_HOME=/home/rust/cargo-home -e OAUTH2_PROVIDER_BASE_URL=$OAUTH2_PROVIDER_BASE_URL -v $PWD/cargo-home-dir:/home/rust/cargo-home:Z -v $PWD/backend:/home/rust/source:Z rust-dev cargo watch -w src -w db -w Cargo.toml -x "run --bin article_server_rs -- -p 5000 --db-path postgresql://$NAME:$NAME@127.0.0.1:5432/$NAME"
podman create --pod $POD_ID -e NODE_ENV=development -e NEXT_TELEMETRY_DISABLED=1 -e BACKEND_URL=http://127.0.0.1:5000 -v $PWD/frontend/:/home/node/source:Z localhost/yarn yarn dev

podman pod start $POD_ID

# https://stackoverflow.com/a/2173421
trap "trap - SIGTERM ; podman pod stop $NAME-dev ; podman pod rm $NAME-dev ; kill -- -$$" SIGINT SIGTERM EXIT

podman pod logs -f $NAME-dev
