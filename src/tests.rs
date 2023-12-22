use crate::*;

#[tokio::test]
async fn simple() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let network = spawn_network().await?;

    println!("🚀🚀🚀 network deployed");

    let honest = network.get_node("honest-0")?;
    let role = honest.reports("node_roles").await?;
    assert_eq!(role as u64, 4);

    let collator_client = get_client(&network, "collator").await?;

    wait_for_block(1, collator_client).await?;

    wait_for_metric(honest, "polkadot_parachain_candidate_disputes_total", 1).await?;

    // let honest_client = honest.client::<subxt::PolkadotConfig>().await?;

    Ok(())
}
