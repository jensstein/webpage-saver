# Web page saver (probably a temporary name)

This is a system to save web pages like pocket, wallabag, shiori, or
many others. The main reason this particular project exists is that I
was using one of the others apps for bookmark saving but had issues with
a particular subset of the pages I wanted to save. I then looked into
the code and that part of the code turned out to be behind multiple
indirections in a not-so-modern programming language. So I decided to
make my own system.

## Backend
The `backend` directory has a server backend is written in Rust using the warp framework.
- Run tests with `scripts/run-cargo-test.sh`. This will run a postgres
  database in a container for use as an integration test database. Both `python3` and `podman` is needed.
- Build using `cargo build`

### Help
```
$ ./article_server_rs -h
article-server

USAGE:
    article_server_rs [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --db-path <database-path>    Path to the database to store webpages in [default: webpages.db]
        --host <host>                Host address to start service on [default: 0.0.0.0]
    -p, --port <port>                Port to start service on [default: 5000]
```

## Frontend
The `frontend` directory contains a web app frontend is written in next.js.
- Build using `yarn install --production --frozen-lockfile && yarn build`

## webextension
The `webextension` directory has a webextension.
- Build using `yarn run parcel build manifest.json --config @parcel/config-webextension`

## android
The `android` directory has an android app. At the moment it can be used
for saving web pages.
- Build using `./gradlew build` or `./gradlew assembleDebug` for the
  debug build.

## License
MIT (<https://spdx.org/licenses/MIT.html>)
