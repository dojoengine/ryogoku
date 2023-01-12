use anyhow::Result;
use clap::{Parser, Subcommand};
use ryogoku_operator::{
    k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    kube::{
        api::{Api, PostParams},
        Client, CustomResourceExt,
    },
    Devnet,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct RyogokuCli {
    #[command(subcommand)]
    command: RyogokuCommand,
}

#[derive(Subcommand)]
enum RyogokuCommand {
    /// Manage Ryogoku CRDs
    Crd {
        #[command(subcommand)]
        command: CrdCommand,
    },
}

#[derive(Subcommand)]
enum CrdCommand {
    /// Print CRD to stdout
    Print,
    /// Install CRD in cluster
    Install {
        /// Submit request but don't persist it
        #[arg(short)]
        dry_run: bool,
    },
}

async fn crd(command: CrdCommand) -> Result<()> {
    match command {
        CrdCommand::Print => {
            let crd = serde_yaml::to_string(&Devnet::crd())?;
            println!("{}", crd);
            Ok(())
        }
        CrdCommand::Install { dry_run } => {
            let client = Client::try_default().await?;
            let crds: Api<CustomResourceDefinition> = Api::all(client);
            let crd_name = Devnet::crd_name();
            let existing = crds.get_opt(crd_name).await?;

            if existing.is_some() {
                println!("CRD {} already exists.", crd_name);
                println!();
                println!("Nothing to do, bye! 🕹");
                return Ok(());
            }

            let opts = PostParams {
                dry_run,
                ..Default::default()
            };

            println!("Creating CRD {}...", crd_name);
            match crds.create(&opts, &Devnet::crd()).await {
                Ok(_) => {
                    println!(" 📦 CRD installed.");
                    println!();
                    println!("Thanks for using Ryogoku 🕹");
                    Ok(())
                }
                Err(err) => {
                    println!(" 🩹 Something went wrong:");
                    println!("Error: {}", err);
                    Ok(())
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = RyogokuCli::parse();

    match cli.command {
        RyogokuCommand::Crd { command } => crd(command).await,
    }
}
