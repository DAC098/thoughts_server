mod error;

mod postgres;
mod migrate;
mod gen_test;

fn commands() -> clap::Command {
    use clap::{Command, Arg, ArgAction};

    Command::new("db")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("migrate")
            .about("database migration operations")
            .subcommand(Command::new("run")
                .about("runs migrates")
                .arg(Arg::new("grouped")
                    .short('g')
                    .long("group")
                    .action(ArgAction::SetTrue)
                    .help("groups migrates into a single transaction"))
                .arg(Arg::new("abort-divergent")
                    .long("continue-divergent")
                    .action(ArgAction::SetFalse)
                    .help("process will continue if divergent migrations are found"))
                .arg(Arg::new("abort-missing")
                    .long("continue-missing")
                    .action(ArgAction::SetFalse)
                    .help("process will continue if missing migrates are found"))
                .arg(postgres::args::connect())
                .arg(postgres::args::user())
                .arg(postgres::args::password())
                .arg(postgres::args::host())
                .arg(postgres::args::port())
                .arg(postgres::args::dbname()))
            .subcommand(Command::new("list")
                .about("lists currently available migrates")
                .arg(postgres::args::connect())
                .arg(postgres::args::user())
                .arg(postgres::args::password())
                .arg(postgres::args::host())
                .arg(postgres::args::port())
                .arg(postgres::args::dbname()))
            .subcommand(Command::new("last-applied")
                .about("shows the last applied migration to the database")
                .arg(postgres::args::connect())
                .arg(postgres::args::user())
                .arg(postgres::args::password())
                .arg(postgres::args::host())
                .arg(postgres::args::port())
                .arg(postgres::args::dbname()))
            .subcommand(Command::new("applied")
                .about("shows all currently applied migrations for the database")
                .arg(postgres::args::connect())
                .arg(postgres::args::user())
                .arg(postgres::args::password())
                .arg(postgres::args::host())
                .arg(postgres::args::port())
                .arg(postgres::args::dbname())))
        .subcommand(Command::new("gen-test")
            .about("generates test data for the connected database"))
}

fn main() -> () {
    let matches = commands().get_matches();

    let result = match matches.subcommand() {
        Some(("migrate", migrate_matches)) => migrate::run(migrate_matches),
        Some(("gen-test", gen_test_matches)) => gen_test::run(gen_test_matches),
        _ => unreachable!()
    };

    if let Err(error) = result {
        if let Some(src) = error.source() {
            eprintln!("Error: {}\n{:#?}", error.message(), src);
        } else {
            eprintln!("Error: {}", error.message());
        }
    }
}
