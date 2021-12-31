# Rattice

[![crates.io](https://img.shields.io/crates/v/rattice.svg)](https://crates.io/crates/rattice/)
[![crates.io](https://img.shields.io/crates/d/rattice)](https://crates.io/crates/rattice/)

A media viewer for web browsers with lattice pattern written in Rust.  
Images and videos are supported.

![screencap](https://raw.githubusercontent.com/oza6ut0ne/rattice/v0.0.3/pic/screencap.png)

Above capture is Rattice running with [MIT-67 Indoor Scene Recognition Dataset](http://web.mit.edu/torralba/www/indoor.html) images for example.

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
Rattice 0.0.3

USAGE:
    rattice [FLAGS] [OPTIONS] [PORT]

FLAGS:
    -e, --eager          Disable lazy image loading [env: RATTICE_EAGER]
    -x, --use-real-ip    Use X-Real-IP as client address in logs [env: RATTICE_USE_REAL_IP]
    -v, --verbose        Increase log level (-v, -vv, -vvv, -vvvv)
    -h, --help           Prints help information
    -V, --version        Prints version information

OPTIONS:
    -b, --bind-address <address>        Bind address [env: RATTICE_BIND_ADDR=]  [default: ::]
    -d, --docroot <docroot>             Specify document root directory [env: RATTICE_DOCROOT=]
    -u, --username <username>           Username for Basic Authentication [env: RATTICE_USER]
    -p, --password <password>           Password for Basic Authentication [env: RATTICE_PASS]
    -r, --random-credencial <length>    Generate random username and/or password with given length
    -s, --server-cert <server-cert>     Server certificate file for HTTPS [env: RATTICE_SERVER_CERT=]
    -k, --server-key <server-key>       Server key file for HTTPS [env: RATTICE_SERVER_KEY=]

ARGS:
    <PORT>    Listen port [env: RATTICE_PORT=]  [default: 3000]
```
