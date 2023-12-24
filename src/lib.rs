use std::time::Duration;

use serde_json::json;
use subxt::{OnlineClient, PolkadotConfig};
use tokio::time::sleep;
use zombienet_sdk::{
    LocalFileSystem, Network, NetworkConfigBuilder, NetworkConfigExt, NetworkNode,
    RegistrationStrategy,
};

#[cfg(test)]
mod tests;

#[subxt::subxt(runtime_metadata_path = "artifacts/polkadot_metadata_small.scale")]
pub mod polkadot {}

pub type Error = Box<dyn std::error::Error>;

pub fn runtime_config() -> serde_json::Value {
    json!({
        "configuration": {
            "config": {
                "max_validators_per_core": 1,
                "needed_approvals": 1,
                "group_rotation_frequency": 10
            }
        }
    })
}

/// Required bins: polkadot, malus, polkadot-parachain
/// built with `--features fast-runtime`
///
/// This spawns a network with 3 honest validators and 1 malicious backer.
/// One parachain with id 2000 and one cumulus-based collator.
pub async fn spawn_network_malus_backer() -> Result<Network<LocalFileSystem>, Error> {
    let network = NetworkConfigBuilder::new()
        .with_relaychain(|r| {
            let patch = runtime_config();

            r.with_chain("westend-local")
                .with_genesis_overrides(patch)
                .with_default_command("polkadot")
                .with_default_args(vec![
                    "--no-hardware-benchmarks".into(),
                    "--insecure-validator-i-know-what-i-do".into(),
                    "-lparachain=debug".into(),
                ])
                .with_node(|node| node.with_name("honest-0"))
                .with_node(|node| node.with_name("honest-1"))
                .with_node(|node| node.with_name("honest-2"))
                .with_node(|node| {
                    node.with_name("malicious-backer")
                        .with_command("malus")
                        .with_subcommand("suggest-garbage-candidate")
                        .with_args(vec![
                            "--no-hardware-benchmarks".into(),
                            "--insecure-validator-i-know-what-i-do".into(),
                            "-lMALUS=trace".into(),
                        ])
                })
        })
        .with_parachain(|p| {
            p.with_id(2000)
                .cumulus_based(true)
                .with_registration_strategy(RegistrationStrategy::InGenesis)
                .with_collator(|n| n.with_name("collator").with_command("polkadot-parachain"))
        })
        .build()
        .unwrap()
        .spawn_native()
        .await?;

    Ok(network)
}

// FIXME: deduplicate this
pub async fn spawn_network_dispute_valid() -> Result<Network<LocalFileSystem>, Error> {
    let network = NetworkConfigBuilder::new()
        .with_relaychain(|r| {
            let patch = runtime_config();

            r.with_chain("westend-local")
                .with_genesis_overrides(patch)
                .with_default_command("polkadot")
                .with_default_args(vec![
                    "--no-hardware-benchmarks".into(),
                    "--insecure-validator-i-know-what-i-do".into(),
                    "-lparachain=debug,parachain::dispute-coordinator=trace".into(),
                ])
                .with_node(|node| node.with_name("honest-0"))
                .with_node(|node| node.with_name("honest-1"))
                .with_node(|node| node.with_name("honest-2"))
                .with_node(|node| node.with_name("honest-3"))
                .with_node(|node| node.with_name("honest-4"))
                .with_node(|node| node.with_name("honest-5"))
                .with_node(|node| node.with_name("honest-6"))
                .with_node(|node| node.with_name("honest-7"))
                .with_node(|node| node.with_name("honest-8"))
                .with_node(|node| {
                    node.with_name("malicious-disputer")
                        .with_command("malus")
                        .with_subcommand("dispute-ancestor")
                        .with_args(vec![
                            "--no-hardware-benchmarks".into(),
                            "--insecure-validator-i-know-what-i-do".into(),
                            "-lMALUS=trace".into(),
                        ])
                })
        })
        .with_parachain(|p| {
            p.with_id(2000)
                .cumulus_based(true)
                .with_registration_strategy(RegistrationStrategy::InGenesis)
                .with_collator(|n| n.with_name("collator").with_command("polkadot-parachain"))
        })
        .build()
        .unwrap()
        .spawn_native()
        .await?;

    Ok(network)
}

pub async fn get_client(
    network: &Network<LocalFileSystem>,
    name: &str,
) -> Result<OnlineClient<PolkadotConfig>, Error> {
    let client = network
        .get_node(name)?
        .client::<subxt::PolkadotConfig>()
        .await?;
    Ok(client)
}

pub async fn wait_for_block(
    number: u32,
    client: OnlineClient<PolkadotConfig>,
) -> Result<(), Error> {
    println!("Waiting for block #{number}:");
    let mut best = client.blocks().subscribe_best().await?;

    while let Some(block) = best.next().await {
        let n = block?.header().number;
        println!("Current best block: #{n}");
        if n >= number {
            break;
        }
    }

    Ok(())
}

pub async fn wait_for_metric(node: &NetworkNode, metric: &str, value: u64) -> Result<(), Error> {
    println!("Waiting for {metric} to reach {value}:");
    loop {
        let current = node.reports(metric).await? as u64;
        println!("{metric} = {current}");
        if current >= value {
            return Ok(());
        }
        sleep(Duration::from_secs(10)).await;
    }
}
