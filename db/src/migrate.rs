use std::fmt::Write;

use clap::ArgMatches;

use crate::error;
use crate::postgres::create_client;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./migrates");
}

fn output_migrate(migrate: &refinery::Migration) -> () {
    let mut output = format!("{} {} {}", migrate.name(), migrate.prefix(), migrate.version());

    if let Some(applied) = migrate.applied_on() {
        write!(&mut output, "\napplied on: {}", applied).unwrap();
    } else {
        write!(&mut output, "\napplied on: unknown").unwrap();
    }

    write!(&mut output, "\nchecksum: {}", migrate.checksum()).unwrap();

    println!("{}", output);
}

fn output_migrate_list(migrates: &Vec<refinery::Migration>) -> () {
    let mut first = true;

    for migrate in migrates {
        if first {
            first = false;
        } else {
            println!("----------------------------------------");
        }

        output_migrate(migrate);
    }
}

pub fn run(args: &ArgMatches) -> error::Result<()> {
    let runner = embedded::migrations::runner();

    match args.subcommand() {
        Some(("run", opts)) => {
            let mut client = create_client(opts)?;
            let report = runner.set_grouped(opts.get_flag("grouped"))
                .set_abort_divergent(opts.get_flag("abort-divergent"))
                .set_abort_missing(opts.get_flag("abort-missing"))
                .run(&mut client)?;

            let applied = report.applied_migrations();

            if applied.len() == 0 {
                println!("no migrates applied");
            } else {
                output_migrate_list(applied);
            }

            client.close()?;
        },
        Some(("list", _opts)) => {
            let migrates = runner.get_migrations();

            if migrates.len() == 0 {
                println!("no migrates found");
            } else {
                output_migrate_list(migrates);
            }
        },
        Some(("last-applied", opts)) => {
            let mut client = create_client(opts)?;

            if let Some(migrate) = runner.get_last_applied_migration(&mut client)? {
                output_migrate(&migrate);
            } else {
                println!("no migration found");
            }

            client.close()?;
        },
        Some(("applied", opts)) => {
            let mut client = create_client(opts)?;
            let migrates = runner.get_applied_migrations(&mut client)?;

            if migrates.len() == 0 {
                println!("no migrates found");
            } else {
                output_migrate_list(&migrates);
            }

            client.close()?;
        },
        _ => unreachable!(),
    }

    Ok(())
}
