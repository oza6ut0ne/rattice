# Rattice

[![crates.io](https://img.shields.io/crates/v/rattice.svg)](https://crates.io/crates/rattice/)
[![crates.io](https://img.shields.io/crates/d/rattice)](https://crates.io/crates/rattice/)

A media viewer for web browsers written in Rust.  
Images and videos are supported.

![screencap](https://raw.githubusercontent.com/oza6ut0ne/rattice/v0.2.0/pic/screencap.png)

*Screenshot of Rattice running with [MIT-67 Indoor Scene Recognition Dataset](http://web.mit.edu/torralba/www/indoor.html) images.*

## Installation

```sh
cargo install rattice
```

or download prebuilt binary from [Releases](https://github.com/oza6ut0ne/rattice/releases).

## Usage

### Quick start

1. Run `rattice` in arbitrary directory.
1. Access [http://localhost:3000/](http://localhost:3000/)

### More options

```shellsession
$ rattice --help
Rattice 0.2.0

USAGE:
    rattice [OPTIONS] [PORT]

ARGS:
    <PORT>    Listen port [env: RATTICE_PORT=] [default: 3000]

OPTIONS:
    -b, --bind-address <ADDRESS>
            Bind address [env: RATTICE_BIND_ADDR=] [default: ::]

    -d, --docroot <DOCROOT>
            Specify document root directory [env: RATTICE_DOCROOT=]

    -s, --sort-by <SORT_BY>
            Sort order [env: RATTICE_SORT_BY=] [default: name] [possible values: name, modified]

    -u, --username <USERNAME>
            Username for Basic Authentication [env: RATTICE_USER]

    -p, --password <PASSWORD>
            Password for Basic Authentication [env: RATTICE_PASS]

    -R, --random-credencial <LENGTH>
            Generate random username and/or password with given length [env:
            RATTICE_RANDOM_CREDENCIAL=]

    -c, --server-cert <SERVER_CERT>
            Server certificate for HTTPS [env: RATTICE_SERVER_CERT=]

    -k, --server-key <SERVER_KEY>
            Server key for HTTPS [env: RATTICE_SERVER_KEY=]

    -t, --title-prefix <TITLE_PREFIX>
            Prefix for HTML title tag [env: RATTICE_TITLE_PREFIX=] [default: Rattice]

FLAGS:
    -r, --reverse        Reverse sort order [env: RATTICE_REVERSE=]
    -e, --eager          Disable lazy image loading [env: RATTICE_EAGER=]
    -x, --use-real-ip    Use X-Real-IP as client address in logs [env: RATTICE_USE_REAL_IP=]
    -v, --verbose        Increase log level (-v, -vv, -vvv, -vvvv)
    -h, --help           Print help information
    -V, --version        Print version information
```

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
