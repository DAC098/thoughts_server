use clap::ArgMatches;

use crate::error;

pub fn run(_args: &ArgMatches) -> error::Result<()> {
    println!("run gen-test");
    Ok(())
}
