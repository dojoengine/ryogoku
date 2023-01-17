use anyhow::Result;
use kube::Client;

use ryogoku_operator::controller;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;
    let controller_task = controller::init(client).await?;

    controller_task.await;
    Ok(())
}
