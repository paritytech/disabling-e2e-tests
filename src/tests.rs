use crate::*;

const DISPUTES_TOTAL_METRIC: &str = "polkadot_parachain_candidate_disputes_total";

#[tokio::test]
async fn test_backing_disabling() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let network = spawn_network_malus_backer().await?;

    println!("🚀🚀🚀 network deployed");

    let honest = network.get_node("honest-0")?;
    let role = honest.reports("node_roles").await?;
    assert_eq!(role as u64, 4);

    let collator_client = get_client(&network, "collator").await?;

    wait_for_block(1, collator_client).await?;

    wait_for_metric(honest, DISPUTES_TOTAL_METRIC, 1).await?;

    let honest_client = honest.client::<subxt::PolkadotConfig>().await?;

    // wait until we have the malicious validator disabled
    loop {
        let call = polkadot::apis().parachain_host().disabled_validators();
        let disabled = honest_client
            .runtime_api()
            .at_latest()
            .await?
            .call(call)
            .await?;
        if disabled.len() == 1 {
            break;
        }
        sleep(Duration::from_secs(5)).await;
    }

    // NOTE: there's a race condition possible
    // after the validator got disabled, but disputes are still ongoing
    // wait for a couple of blocks to avoid it
    sleep(Duration::from_secs(12)).await;

    // get the current disputes metric
    let total_disputes = honest.reports(DISPUTES_TOTAL_METRIC).await? as u64;

    // wait a bit
    sleep(Duration::from_secs(120)).await;

    let new_total_disputes = honest.reports(DISPUTES_TOTAL_METRIC).await? as u64;

    // ensure that no new disputes were created after validator got disabled
    assert_eq!(total_disputes, new_total_disputes);

    Ok(())
}

#[tokio::test]
async fn test_disputes_offchain_disabling() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let network = spawn_network_dispute_valid().await?;

    println!("🚀🚀🚀 network deployed");

    let honest = network.get_node("honest-0")?;
    let role = honest.reports("node_roles").await?;
    assert_eq!(role as u64, 4);

    let collator_client = get_client(&network, "collator").await?;

    wait_for_block(1, collator_client).await?;

    wait_for_metric(honest, DISPUTES_TOTAL_METRIC, 1).await?;

    // NOTE: there's a race condition possible
    // after the dispute concluded and before the validator got disabled
    // wait for a block to avoid it
    sleep(Duration::from_secs(6)).await;

    // get the current disputes metric
    let total_disputes = honest.reports(DISPUTES_TOTAL_METRIC).await? as u64;

    // wait a bit
    sleep(Duration::from_secs(120)).await;

    let new_total_disputes = honest.reports(DISPUTES_TOTAL_METRIC).await? as u64;

    // ensure that no new disputes were created after validator got disabled offchain
    assert_eq!(total_disputes, new_total_disputes);

    Ok(())
}
