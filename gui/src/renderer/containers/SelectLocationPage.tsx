import { connect } from 'react-redux';

import BridgeSettingsBuilder from '../../shared/bridge-settings-builder';
import { LiftedConstraint, Ownership, RelayLocation } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import SelectLocation from '../components/SelectLocation';
import withAppContext, { IAppContext } from '../context';
import { createWireguardRelayUpdater } from '../lib/constraint-updater';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, props: IHistoryProps & IAppContext) => {
  let selectedExitLocation: RelayLocation | undefined;
  let selectedEntryLocation: RelayLocation | undefined;
  let selectedBridgeLocation: LiftedConstraint<RelayLocation> | undefined;
  let multihopEnabled = false;

  if ('normal' in state.settings.relaySettings) {
    const exitLocation = state.settings.relaySettings.normal.location;
    if (exitLocation !== 'any') {
      selectedExitLocation = exitLocation;
    }
  }

  const relaySettings = state.settings.relaySettings;
  const tunnelProtocol = 'normal' in relaySettings ? relaySettings.normal.tunnelProtocol : 'any';

  if (tunnelProtocol === 'openvpn' && 'normal' in state.settings.bridgeSettings) {
    selectedBridgeLocation = state.settings.bridgeSettings.normal.location;
  } else if ('normal' in relaySettings) {
    multihopEnabled = relaySettings.normal.wireguard.useMultihop;

    const entryLocation = relaySettings.normal.wireguard.entryLocation;
    if (multihopEnabled && entryLocation !== 'any') {
      selectedEntryLocation = entryLocation;
    }
  }

  const allowEntrySelection =
    (tunnelProtocol === 'openvpn' && state.settings.bridgeState === 'on') ||
    ((tunnelProtocol === 'any' || tunnelProtocol === 'wireguard') && multihopEnabled);

  const providers = 'normal' in relaySettings ? relaySettings.normal.providers : [];
  const ownership = 'normal' in relaySettings ? relaySettings.normal.ownership : Ownership.any;

  return {
    locale: state.userInterface.locale,
    selectedExitLocation,
    selectedEntryLocation,
    selectedBridgeLocation,
    relayLocations: filterLocations(state.settings.relayLocations, providers, ownership),
    bridgeLocations: filterLocations(state.settings.bridgeLocations, providers, ownership),
    allowEntrySelection,
    tunnelProtocol,
    providers,
    ownership,

    onSelectEntryLocation: async (entryLocation: RelayLocation) => {
      // dismiss the view first
      props.history.dismiss();

      const relayUpdate = createWireguardRelayUpdater(state.settings.relaySettings)
        .tunnel.wireguard((wireguard) => wireguard.entryLocation.exact(entryLocation))
        .build();

      try {
        await props.app.updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to select the entry location', error.message);
      }
    },
  };
};
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onClose: () => props.history.dismiss(),
    onViewFilter: () => props.history.push(RoutePath.filter),
    onSelectExitLocation: async (relayLocation: RelayLocation) => {
      // dismiss the view first
      props.history.dismiss();

      try {
        const relayUpdate = RelaySettingsBuilder.normal().location.fromRaw(relayLocation).build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.connectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the exit location: ${error.message}`);
      }
    },
    onSelectBridgeLocation: async (bridgeLocation: RelayLocation) => {
      // dismiss the view first
      props.history.dismiss();

      try {
        await props.app.updateBridgeSettings(
          new BridgeSettingsBuilder().location.fromRaw(bridgeLocation).build(),
        );
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the bridge location: ${error.message}`);
      }
    },
    onSelectClosestToExit: async () => {
      // dismiss the view first
      props.history.dismiss();

      try {
        await props.app.updateBridgeSettings(new BridgeSettingsBuilder().location.any().build());
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to set the bridge location to closest to exit: ${error.message}`);
      }
    },
    onClearProviders: async () => {
      await props.app.updateRelaySettings({ normal: { providers: [] } });
    },
    onClearOwnership: async () => {
      await props.app.updateRelaySettings({ normal: { ownership: Ownership.any } });
    },
  };
};

function filterLocations(
  locations: IRelayLocationRedux[],
  providers: string[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  const locationsFilteredByOwnership = filterLocationsByOwnership(locations, ownership);
  const locationsFilteredByProvider = filterLocationsByProvider(
    locationsFilteredByOwnership,
    providers,
  );

  return locationsFilteredByProvider;
}

function filterLocationsByOwnership(
  locations: IRelayLocationRedux[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  if (ownership === Ownership.any) {
    return locations;
  }

  const expectOwned = ownership === Ownership.mullvadOwned;
  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({
          ...city,
          relays: city.relays.filter((relay) => relay.owned === expectOwned),
        }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}

function filterLocationsByProvider(
  locations: IRelayLocationRedux[],
  providers: string[],
): IRelayLocationRedux[] {
  if (providers.length === 0) {
    return locations;
  }

  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({
          ...city,
          relays: city.relays.filter((relay) => providers.includes(relay.provider)),
        }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(SelectLocation)),
);
