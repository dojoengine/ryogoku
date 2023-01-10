use anyhow::Result;
use clap::{command, ArgMatches, Command};
use dojo_operator::{CustomResourceExt, Devnet};

fn crd(_matches: &ArgMatches) -> Result<()> {
    let crd = serde_yaml::to_string(&Devnet::crd())?;
    println!("{}", crd);
    Ok(())
}

fn main() -> Result<()> {
    let matches = command!()
        .subcommand_required(true)
        .subcommand(Command::new("crd").about("Print Dojo CRDs"))
        .get_matches();

    match matches.subcommand() {
        Some(("crd", crd_matches)) => crd(crd_matches),
        _ => unreachable!(),
    }
}
