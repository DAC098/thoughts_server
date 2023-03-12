use std::str::FromStr;

use postgres::{Client, Config, NoTls};
use clap::ArgMatches;

use crate::error;

pub mod args {
    use clap::{Arg, ArgAction};

    pub fn connect() -> Arg {
        Arg::new("connect")
            .short('c')
            .long("connect")
            .action(ArgAction::Set)
            .help("connection string for postgres")
            .conflicts_with_all([
                "user",
                "password",
                "host",
                "port",
                "dbname",
            ])
    }

    pub fn user() -> Arg {
        Arg::new("user")
            .short('u')
            .long("user")
            .action(ArgAction::Set)
            .help("user for postgres connection. defaults to \"postgres\"")
            .conflicts_with("connect")
    }

    pub fn password() -> Arg {
        Arg::new("password")
            .short('p')
            .long("password")
            .action(ArgAction::Set)
            .help("password for postgres connection. defaults to \"\"")
            .conflicts_with("connect")
    }

    pub fn host() -> Arg {
        Arg::new("host")
            .long("host")
            .action(ArgAction::Set)
            .help("host for postgres connection. defaults to \"localhost\"")
            .conflicts_with("connect")
    }

    pub fn port() -> Arg {
        Arg::new("port")
            .short('P')
            .long("port")
            .action(ArgAction::Set)
            .help("port for postgres connection. defaults to 5432")
            .conflicts_with("connect")
    }

    pub fn dbname() -> Arg {
        Arg::new("dbname")
            .long("dbname")
            .action(ArgAction::Set)
            .help("dbname for postgres connection. defaults to \"thoughts\"")
            .conflicts_with("connect")
    }
}

pub fn create_client(args: &ArgMatches) -> error::Result<Client> {
    if let Some(connect) = args.get_one::<String>("connect") {
        Ok(Config::from_str(connect.as_str())?.connect(NoTls)?)
    } else {
        let mut config = Config::new();

        if let Some(user) = args.get_one::<String>("user") {
            config.user(user.as_str());
        } else {
            config.user("postgres");
        }

        if let Some(password) = args.get_one::<String>("password") {
            config.password(password.as_bytes());
        } else {
            config.password("");
        }

        if let Some(host) = args.get_one::<String>("host") {
            config.host(host.as_str());
        } else {
            config.host("localhost");
        }

        if let Some(port_str) = args.get_one::<String>("port") {
            let Ok(port) = u16::from_str(port_str) else {
                return Err(error::Error::new()
                    .with_message("invalid port number provided"));
            };

            config.port(port);
        } else {
            config.port(5432);
        }

        if let Some(dbname) = args.get_one::<String>("dbname") {
            config.dbname(dbname.as_str());
        } else {
            config.dbname("thoughts");
        }

        Ok(config.connect(NoTls)?)
    }
}
