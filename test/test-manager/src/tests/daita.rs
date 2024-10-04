use anyhow::{anyhow, bail, ensure, Context};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::{
    relay_constraints::GeographicLocationConstraint, relay_list::RelayEndpointData,
    states::TunnelState,
};
use talpid_types::{net::TunnelEndpoint, tunnel::ErrorStateCause};
use test_macro::test_function;
use test_rpc::ServiceClient;

use super::{helpers, Error, TestContext};

/// Test that daita and daita_direct_only works by connecting
/// - to a non-DAITA relay with singlehop (should block)
/// - to a DAITA relay with singlehop
/// - to a DAITA relay with auto-multihop by disabling direct_only
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
        .relays()
        .flat_map(|relay| match &relay.endpoint_data {
            RelayEndpointData::Wireguard(wireguard) => Some((relay, wireguard)),
            _ => None,
        });

    // Select two relays to use for the test, one with DAITA and one without.
    let daita_relay = wg_relays
        .clone()
        .find(|(_relay, wireguard_data)| wireguard_data.daita)
        .map(|(relay, _)| relay)
        .context("Failed to find a daita wireguard relay")?;
    log::info!("Selected daita relay: {}", daita_relay.hostname);
    let daita_relay_location = GeographicLocationConstraint::hostname(
        &daita_relay.location.country_code,
        &daita_relay.location.city_code,
        &daita_relay.hostname,
    );

    let non_daita_relay = wg_relays
        .clone()
        .find(|(_relay, wireguard_data)| !wireguard_data.daita)
        .map(|(relay, _)| relay)
        .context("Failed to find a non-daita wireguard relay")?;
    let non_daita_relay_location = GeographicLocationConstraint::hostname(
        &non_daita_relay.location.country_code,
        &non_daita_relay.location.city_code,
        &non_daita_relay.hostname,
    );
    log::info!("Selected non-daita relay: {}", non_daita_relay.hostname);

    let non_daita_location_query = RelayQueryBuilder::new()
        .wireguard()
        .location(non_daita_relay_location.clone())
        .build();

    let daita_location_query = RelayQueryBuilder::new()
        .wireguard()
        .location(daita_relay_location.clone())
        .build();

    let daita_to_non_daita_multihop_query = RelayQueryBuilder::new()
        .wireguard()
        .multihop()
        .entry(daita_relay_location.clone())
        .location(non_daita_relay_location.clone())
        .build();

    let non_daita_multihop_query = RelayQueryBuilder::new()
        .wireguard()
        .multihop()
        .entry(non_daita_relay_location.clone())
        .build();

    let mut events = mullvad_client
        .events_listen()
        .await?
        .inspect(|event| log::debug!("New daemon event: {event:?}"));

    log::info!("Connecting to non-daita relay with DAITA by automatically using multihop");
    {
        helpers::set_relay_settings(&mut mullvad_client, non_daita_location_query.clone()).await?;
        mullvad_client.set_enable_daita(true).await?;
        mullvad_client.connect_tunnel().await?;
        let state = wait_for_daemon_reconnect(&mut events)
            .await
            .context("Failed to connect with 'direct only' disabled")?;

        let endpoint: &TunnelEndpoint = state.endpoint().ok_or(anyhow!("No endpoint"))?;
        ensure!(endpoint.daita, "DAITA must be used");
        ensure!(endpoint.entry_endpoint.is_some(), "multihop must be used");

        log::info!("Successfully multihopped with 'direct only' disabled");
    }

    log::info!("Connecting to non-daita relay with 'DAITA: direct only'");
    {
        mullvad_client.set_daita_direct_only(true).await?;

        let result = wait_for_daemon_reconnect(&mut events).await;
        let Err(Error::UnexpectedErrorState(state)) = result else {
            bail!("Connection failed unsuccessfully, reason: {:?}", result);
        };
        let ErrorStateCause::TunnelParameterError(_) = state.cause() else {
            bail!("Connection failed unsuccessfully, cause: {}", state.cause());
        };

        log::info!("Failed to connect, this is expected!");
    }

    log::info!("Connecting to daita relay with 'direct_only' disabled");
    {
        helpers::set_relay_settings(&mut mullvad_client, daita_location_query).await?;

        let state = wait_for_daemon_reconnect(&mut events)
            .await
            .context("Failed to connect to daita location with 'direct_only' disabled")?;

        let endpoint = state.endpoint().context("No endpoint")?;
        ensure!(endpoint.daita, "DAITA must be used");
        ensure!(
            endpoint.entry_endpoint.is_none(),
            "multihop must not be used"
        );

        log::info!("Successfully singlehopped with 'direct_only' disabled");
    }

    log::info!("Connecting to daita relay with multihop");
    {
        helpers::set_relay_settings(&mut mullvad_client, daita_to_non_daita_multihop_query).await?;
        let state = wait_for_daemon_reconnect(&mut events)
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
        let result = wait_for_daemon_reconnect(&mut events).await;
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

async fn wait_for_daemon_reconnect(
    mut event_stream: impl futures::Stream<Item = Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
) -> Result<TunnelState, Error> {
    // wait until the daemon informs us that it's trying to connect
    helpers::find_daemon_event(&mut event_stream, |event| match event {
        DaemonEvent::TunnelState(state) => Some(match state {
            TunnelState::Connecting { .. } => Ok(state),
            TunnelState::Connected { .. } => return None,
            TunnelState::Disconnecting { .. } => return None,
            TunnelState::Disconnected { .. } => Err(Error::UnexpectedTunnelState(Box::new(state))),
            TunnelState::Error(state) => Err(Error::UnexpectedErrorState(state)),
        }),
        _ => None,
    })
    .await??;

    // then wait until the daemon informs us that it connected (or failed)
    helpers::find_daemon_event(&mut event_stream, |event| match event {
        DaemonEvent::TunnelState(state) => match state {
            TunnelState::Connecting { .. } => None,
            TunnelState::Connected { .. } => Some(Ok(state)),
            _ => Some(Err(Error::UnexpectedTunnelState(Box::new(state)))),
        },
        _ => None,
    })
    .await?
}
