TurboBunny
==========

Turbobunny is a bunny1/bunnylol clone. It's a quicklinking tool that's meant to
stand in for your browser's default search engine and provide some convenient
aliases for stuff you do a lot. Ideally it should feel kinda like having a nice
set of shell rcscripts for the web.

It's written as an actix-web server, with a handful of routes and a couple
static resources:

* `/cmd?q={}` is where the actual traffic goes
* `/`, `/index`, `/index.html`, `/list`, and so on are where the metadata lives
* `/static` serves up the contents of the `./resources` directory

There are a couple others where specific paths are required for things like
favicons and OpenSearch hooks -- look at main.rs for how routes are assigned.


Developing
==========

TurboBunny supports listenfd as well as standalone port binding, so you can
set it up with live recompiles and one open port, for easy dev work. To get that
going, you need two utilities cargo can install for you:

```bash
cargo install systemfd cargo-watch
```

* systemfd opens a socket and hands it off to a subordinate process, imitating
  the way systemd-provided sockets work on linux machines.
* cargo-watch monitors your source tree and runs a command on changes.

Putting these together:

```bash
systemfd --no-pid -s http::8080 -- cargo watch -x "run --bin turbobunny -- localhost:8080"
```

That'll give you automatic recompiles/restarts whenever the code changes.
