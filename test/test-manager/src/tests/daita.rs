use anyhow::{anyhow, bail, ensure, Context};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::relay_list::RelayEndpointData;
use talpid_types::{net::TunnelEndpoint, tunnel::ErrorStateCause};
use test_macro::test_function;
use test_rpc::ServiceClient;

use super::{helpers, Error, TestContext};

/// Test that daita and daita_smart_routing works by connecting
/// - to a non-DAITA relay with singlehop (should block)
/// - to a DAITA relay with singlehop
/// - to a DAITA relay with auto-multihop using smart_routing
/// - to a DAITA relay with explicit multihop
/// - to a non-DAITA relay with multihop (should block)
///
/// # Limitations
///
/// The test does not analyze any traffic, nor verify that DAITA is in use in any way except
/// by looking at [TunnelEndpoint::daita].
#[test_function]
pub async fn test_daita(
    _ctx: TestContext,
    _rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let relay_list = mullvad_client.get_relay_locations().await?;
    let wg_relays = relay_list
        .countries
        .iter()
        .flat_map(|countries| &countries.cities)
        .flat_map(|cities| &cities.relays)
        .flat_map(|relays| match &relays.endpoint_data {
            RelayEndpointData::Wireguard(wireguard) => Some((relays, wireguard)),
            _ => None,
        });

    // Select two relays to use for hte test, one with DAITA and one without.
    let daita_relay = wg_relays
        .clone()
        .find(|(_relay, wireguard_data)| wireguard_data.daita)
        .map(|(relay, _)| relay)
        .context("Failed to find a daita wireguard relay")?;
    log::info!("Selected daita relay: {}", daita_relay.hostname);

    let non_daita_relay = wg_relays
        .clone()
        .find(|(_relay, wireguard_data)| !wireguard_data.daita)
        .map(|(relay, _)| relay)
        .context("Failed to find a non-daita wireguard relay")?;
    log::info!("Selected non-daita relay: {}", non_daita_relay.hostname);

    let mut non_daita_location_query = RelayQueryBuilder::new().wireguard().build();
    non_daita_location_query.location = helpers::into_constraint(non_daita_relay);

    let mut daita_location_query = RelayQueryBuilder::new().wireguard().build();
    daita_location_query.location = helpers::into_constraint(daita_relay);

    let mut daita_to_non_daita_multihop_query =
        RelayQueryBuilder::new().wireguard().multihop().build();
    daita_to_non_daita_multihop_query
        .wireguard_constraints
        .entry_location = helpers::into_constraint(daita_relay);
    daita_to_non_daita_multihop_query.location = helpers::into_constraint(non_daita_relay);

    let mut non_daita_multihop_query = RelayQueryBuilder::new().wireguard().multihop().build();
    non_daita_multihop_query
        .wireguard_constraints
        .entry_location = helpers::into_constraint(non_daita_relay);

    log::info!("Enabling DAITA and trying to connect without smart_routing");
    {
        mullvad_client.set_enable_daita(true).await?;
        mullvad_client.set_daita_smart_routing(false).await?;
        helpers::set_relay_settings(&mut mullvad_client, non_daita_location_query).await?;

        let result = helpers::connect_and_wait(&mut mullvad_client).await;
        let Err(Error::UnexpectedErrorState(state)) = result else {
            bail!("Connection failed unsuccessfully, reason: {:?}", result);
        };
        let ErrorStateCause::TunnelParameterError(_) = state.cause() else {
            bail!("Connection failed unsuccessfully, cause: {}", state.cause());
        };

        log::info!("Failed to connect, this is expected!");
    }

    log::info!("Connecting to non-daita relay with use smart_routing");
    {
        helpers::disconnect_and_wait(&mut mullvad_client).await?;
        mullvad_client.set_daita_smart_routing(true).await?;
        let state = helpers::connect_and_wait(&mut mullvad_client)
            .await
            .context("Failed to connect with smart_routing enabled")?;

        let endpoint: &TunnelEndpoint = state.endpoint().ok_or(anyhow!("No endpoint"))?;
        ensure!(endpoint.daita, "DAITA must be used");
        ensure!(endpoint.entry_endpoint.is_some(), "multihop must be used");

        log::info!("Successfully multihopped with use smart_routing");
    }

    log::info!("Connecting to daita relay with smart_routing");
    {
        helpers::set_relay_settings(&mut mullvad_client, daita_location_query).await?;

        // TODO: connect_and_wait has a timing issue where if we're already connected, it might
        // pick up on the previous connect state before the daemon can do its thing
        let state = helpers::connect_and_wait(&mut mullvad_client)
            .await
            .context("Failed to connect to daita location with smart_routing enabled")?;

        let endpoint = state.endpoint().context("No endpoint")?;
        ensure!(endpoint.daita, "DAITA must be used");
        ensure!(
            endpoint.entry_endpoint.is_none(),
            "multihop must not be used"
        );

        log::info!("Successfully singlehopped with smart_routing");
    }

    log::info!("Connecting to daita relay with multihop");
    {
        helpers::set_relay_settings(&mut mullvad_client, daita_to_non_daita_multihop_query).await?;
        let state = helpers::connect_and_wait(&mut mullvad_client)
            .await
            .context("Failed to connect via daita location with multihop enabled")?;

        let endpoint = state.endpoint().context("No endpoint")?;
        ensure!(endpoint.daita, "DAITA must be used");
        ensure!(endpoint.entry_endpoint.is_some(), "multihop must be used");

        log::info!("Successfully connected with multihop");
    }

    log::info!("Connecting to non_daita relay with multihop");
    {
        helpers::set_relay_settings(&mut mullvad_client, non_daita_multihop_query).await?;
        let result = helpers::connect_and_wait(&mut mullvad_client).await;
        let Err(Error::UnexpectedErrorState(state)) = result else {
            bail!("Connection failed unsuccessfully, reason: {:?}", result);
        };
        let ErrorStateCause::TunnelParameterError(_) = state.cause() else {
            bail!("Connection failed unsuccessfully, cause: {}", state.cause());
        };

        log::info!("Failed to connect, this is expected!");
    }

    Ok(())
}
