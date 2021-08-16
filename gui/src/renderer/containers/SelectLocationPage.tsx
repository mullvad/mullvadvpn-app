import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import BridgeSettingsBuilder from '../../shared/bridge-settings-builder';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import SelectLocation from '../components/SelectLocation';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { IRelayLocationRedux, RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';
import userInterfaceActions from '../redux/userinterface/actions';
import { LocationScope } from '../redux/userinterface/reducers';

const mapStateToProps = (state: IReduxState) => {
  let selectedExitLocation: RelayLocation | undefined;
  let selectedBridgeLocation: LiftedConstraint<RelayLocation> | undefined;

  if ('normal' in state.settings.relaySettings) {
    const exitLocation = state.settings.relaySettings.normal.location;
    if (exitLocation !== 'any') {
      selectedExitLocation = exitLocation;
    }
  }

  if ('normal' in state.settings.bridgeSettings) {
    selectedBridgeLocation = state.settings.bridgeSettings.normal.location;
  }

  const allowBridgeSelection = state.settings.bridgeState === 'on';
  const locationScope = allowBridgeSelection
    ? state.userInterface.locationScope
    : LocationScope.relay;

  return {
    selectedExitLocation,
    selectedBridgeLocation,
    relayLocations: filterLocationsByProvider(
      state.settings.relayLocations,
      state.settings.relaySettings,
    ),
    bridgeLocations: state.settings.bridgeLocations,
    locationScope,
    allowBridgeSelection,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  const userInterface = bindActionCreators(userInterfaceActions, dispatch);

  return {
    onClose: () => props.history.dismiss(),
    onViewFilterByProvider: () => props.history.push(RoutePath.filterByProvider),
    onChangeLocationScope: (scope: LocationScope) => {
      userInterface.setLocationScope(scope);
    },
    onSelectExitLocation: async (relayLocation: RelayLocation) => {
      // dismiss the view first
      props.history.dismiss();

      try {
        const relayUpdate = RelaySettingsBuilder.normal().location.fromRaw(relayLocation).build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.connectTunnel();
      } catch (e) {
        log.error(`Failed to select the exit location: ${e.message}`);
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
        log.error(`Failed to select the bridge location: ${e.message}`);
      }
    },
    onSelectClosestToExit: async () => {
      // dismiss the view first
      props.history.dismiss();

      try {
        await props.app.updateBridgeSettings(new BridgeSettingsBuilder().location.any().build());
      } catch (e) {
        log.error(`Failed to set the bridge location to closest to exit: ${e.message}`);
      }
    },
  };
};

function filterLocationsByProvider(
  locations: IRelayLocationRedux[],
  relaySettings: RelaySettingsRedux,
): IRelayLocationRedux[] {
  const providers =
    'normal' in relaySettings && relaySettings.normal.providers.length > 0
      ? relaySettings.normal.providers
      : undefined;

  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({
          ...city,
          relays: city.relays.filter((relay) => providers?.includes(relay.provider) ?? true),
        }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(SelectLocation)),
);
