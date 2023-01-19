mod devnet;

use anyhow::Result;
use clap::{Parser, Subcommand};
use devnet::DevnetOut;
use ryogoku_operator::{
    k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    kube::{
        api::{Api, DeleteParams, ListParams, ObjectMeta, PostParams},
        Client, CustomResourceExt, ResourceExt,
    },
    Devnet, DevnetSpec,
};
use tabled::{Style, Table};

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
    /// Manage development networks.
    Devnet {
        #[command(subcommand)]
        command: DevnetCommand,
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

#[derive(Subcommand)]
enum DevnetCommand {
    /// Create a new development network.
    //
    // TODO: add -n flag.
    Create {
        /// Network name.
        name: String,
    },
    /// List all development networks.
    //
    // TODO: add -A and -n flags.
    List,
    //
    // TODO: add -n flag.
    Delete {
        /// Network name.
        name: String,
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

async fn devnet(command: DevnetCommand) -> Result<()> {
    let client = Client::try_default().await?;
    match command {
        DevnetCommand::Create { name } => {
            // TODO: client based on namespace flag.
            let devnets: Api<Devnet> = Api::namespaced(client, "default");
            let data = Devnet {
                metadata: ObjectMeta {
                    name: Some(name),
                    namespace: Some("default".to_string()),
                    ..ObjectMeta::default()
                },
                spec: DevnetSpec::default(),
                status: None,
            };
            let devnet = devnets.create(&PostParams::default(), &data).await?;
            println!("devnet {} created", devnet.name_any());
            Ok(())
        }
        DevnetCommand::List => {
            // TODO: client based on namespace flag.
            let devnets: Api<Devnet> = Api::all(client);
            let all = devnets.list(&ListParams::default()).await?;

            let all_out: Vec<_> = all.into_iter().map(DevnetOut::new).collect();
            let table = Table::new(all_out).with(Style::empty()).to_string();
            println!("{}", table);
            Ok(())
        }
        DevnetCommand::Delete { name } => {
            // TODO: client based on namespace flag.
            let devnets: Api<Devnet> = Api::namespaced(client, "default");
            let dp = DeleteParams::default();
            let _devnet = devnets.delete(&name, &dp).await?;
            println!("devnet {} deleted", name);
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = RyogokuCli::parse();

    match cli.command {
        RyogokuCommand::Crd { command } => crd(command).await,
        RyogokuCommand::Devnet { command } => devnet(command).await,
    }
}
