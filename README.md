# Thoughts Server

A journalling server of sorts that allows you to record various forms of data for yourself and then allow other users to then view that information.

## Background

This effectively started out with me having various texts files on my computer that was me getting the thoughts out of my head and on to "paper". As it went on I started to record various other things like general mood on a scale of 1 - 10. At the request of others, they wanted to see some of this information but I did not have an easy way to do that and I was also looking for something to keep me entertained for a while. So as any sane person would do, I decided to build a web server that would house this information and allow them to access it and easily view things.

## Features

Current features with plans to add more

 - [x] Daily entries that store text
 - [x] Custom fields that allow you to store integers, integer ranges, floats, float ranges, time, and time ranges along with a description of what the field is.
 - [x] Custom tags with colors that can be assigned to an entry
 - [x] Controlled user accounts so that only those you want to have access to the server do.
 - [x] Graphing for the custom fields to have a more visual representation of the data
 - [x] Change account information such as username, full name, password, email, etc.
 - [x] Search entries over a given date range and show / hide fields as you wish
 - [ ] Password recovery in case you forgot it
 - [ ] Mobile compatable interface for ease of use

more features to come as the server progresses with some planned in the futre.

 - Assign custom fields to a user at the request of another
 - Include more medical related information that some could write down for another user
 - Better user accessability
 - Comments on another users information
 - More information regarding a user if used in a medical environment
 - Other things?

## Technical Details

The main server is written in Rust and uses the Actix framework to handle the server capabilities. PostgreSQL is use for the database and uses Tokio Postgres for communication.

React front-end using the Fluent UI as a framework. React-Router for page navigation and rendering. Redux Toolkit for global state management.

 - Rust `1.51.0`
 - OpenSSL `1.1.1f`
 - PostgreSQL `13`
 - NodeJS `15.14.0`

Rust package versions can be found in the `Cargo.toml`
NodeJS package version can be found in the `package.json`

currently been building on Ubuntu Server `20.04` so no testing has been done on Windows or MacOS. Over versions of various packages and software may work but no testing has been done to determine those versions.

No official server version as I am unsure about what to set it currently.

### Building

This will require the OpenSSL libraries and header files. The [docs](https://docs.rs/openssl/0.10.34/openssl/) for the rust package talks about how to download the headers and libraries.

As stated above this has currently only been built on Ubuntu Linux. Once you have the OpenSSL requirements run
```bash
# development
$ cargo build --workspace --bins
# release
$ cargo build --workspace --bins --release
```

The server should be ready to go by this point

### Running

Command line arguments are currently limited. You will need to specify configuration files to setup the server. Specifying multiple config files and will be loaded in the order given with later files overriding earlier values. A configuration is as follows:

```yaml
bind:
    # valid ipv4 or ipv6 address for the system
    - host: "0.0.0.0"
      port: 8080
      
    - host: "::1"
# default port if a bind host does not have a port specified
port: 8080

# defaults to system max
threads: 16

# default values for the actix web framework being used
backlog: 2048
max_connections: 25000
max_connection_rate: 256

db:
    username: "postgres"
    password: "password"
    database: "thoughts"
    port: 5432
    hostname: "localhost"

session:
    # used in cookie sessions to restrict cookies to a
    # specific domain
    domain: ""

email:
    enabled: false
    from: "no_reply@example.com"
    username: "no_reply@example.com"
    password: "password"
    relay: "smtp.google.com"

# ssl information for secure server connections. if enabled
# then both key and cert must be provided and be valid files.
ssl:
    enable: false
    key: null
    cert: null
```

### Database

Currently only PostgreSQL is supported. If you run your own or have one running there is not easy way to create the database in that instance (work in progress). You can use Docker and Docker Compose to start one from the project root which will create and setup the database for use by the server.

```bash
# should create and start the dabase
$ docker-compose up
# start the database if it was already created
$ docker-compose start
```

## Contributions

No idea. If you are interested in helping out with this then sweet!

## THIS IS A WORK IN PROGRESS
## PLEASE DO NOT USE THIS FOR PRODUCTION PURPOSES YET

First personal project like this so bear with me as I figure this stuff out.