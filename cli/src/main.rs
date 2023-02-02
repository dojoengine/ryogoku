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
    Create {
        /// Network name.
        name: String,
        /// Create the network in the given namespace.
        #[arg(short, long)]
        namespace: Option<String>,
        /// Specify the service type.
        #[arg(short, long)]
        service_type: Option<String>,
        /// Shorthand for `--service-type=LoadBalancer`.
        #[arg(short, long)]
        expose: bool,
    },
    /// List all development networks.
    List {
        /// List networks in the given namespace.
        #[arg(short, long)]
        namespace: Option<String>,
        /// If present, list networks in all namespaces.
        #[arg(short = 'A', long)]
        all_namespaces: bool,
    },
    /// Delete a development network.
    Delete {
        /// Network name.
        name: String,
        /// Delete the network in the given namespace.
        #[arg(short, long)]
        namespace: Option<String>,
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
        DevnetCommand::Create {
            name,
            namespace,
            service_type,
            expose,
        } => {
            let namespace = namespace.unwrap_or_else(|| "default".to_string());
            let devnets: Api<Devnet> = Api::namespaced(client, &namespace);

            let service_type = if service_type.is_some() {
                service_type
            } else if expose {
                Some("NodePort".to_string())
            } else {
                None
            };
            let data = Devnet {
                metadata: ObjectMeta {
                    name: Some(name),
                    namespace: Some(namespace),
                    ..ObjectMeta::default()
                },
                spec: DevnetSpec {
                    service_type,
                    ..DevnetSpec::default()
                },
                status: None,
            };
            let devnet = devnets.create(&PostParams::default(), &data).await?;

            println!("devnet {} created", devnet.name_any());

            Ok(())
        }
        DevnetCommand::List {
            namespace,
            all_namespaces,
        } => {
            let devnets: Api<Devnet> = if all_namespaces {
                Api::all(client)
            } else {
                let namespace = namespace.unwrap_or_else(|| "default".to_string());
                Api::namespaced(client, &namespace)
            };

            let all = devnets.list(&ListParams::default()).await?;

            let all_out: Vec<_> = all.into_iter().map(DevnetOut::new).collect();

            let table = Table::new(all_out).with(Style::empty()).to_string();
            println!("{}", table);

            Ok(())
        }
        DevnetCommand::Delete { name, namespace } => {
            let namespace = namespace.unwrap_or_else(|| "default".to_string());
            let devnets: Api<Devnet> = Api::namespaced(client, &namespace);

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
