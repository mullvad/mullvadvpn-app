import { goBack } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { RelayLocation } from '../../shared/daemon-rpc-types';
import SelectLocation from '../components/SelectLocation';
import RelaySettingsBuilder from '../lib/relay-settings-builder';
import userInterfaceActions from '../redux/userinterface/actions';
import { LocationScope } from '../redux/userinterface/reducers';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => {
  let selectedExitLocation: RelayLocation | undefined;
  let selectedBridgeLocation: RelayLocation | undefined;

  if ('normal' in state.settings.relaySettings) {
    const exitLocation = state.settings.relaySettings.normal.location;
    if (exitLocation !== 'any') {
      selectedExitLocation = exitLocation;
    }
  }

  if ('normal' in state.settings.bridgeSettings) {
    const bridgeLocation = state.settings.bridgeSettings.normal.location;
    if (bridgeLocation !== 'any') {
      selectedBridgeLocation = bridgeLocation;
    }
  }

  const allowBridgeSelection = state.settings.bridgeState === 'on';
  const locationScope = allowBridgeSelection
    ? state.userInterface.locationScope
    : LocationScope.relay;

  return {
    selectedExitLocation,
    selectedBridgeLocation,
    relayLocations: state.settings.relayLocations,
    bridgeLocations: state.settings.bridgeLocations,
    locationScope,
    allowBridgeSelection,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  const userInterface = bindActionCreators(userInterfaceActions, dispatch);

  return {
    onClose: () => history.goBack(),
    onChangeLocationScope: (scope: LocationScope) => {
      userInterface.setLocationScope(scope);
    },
    onSelectExitLocation: async (relayLocation: RelayLocation) => {
      // dismiss the view first
      history.goBack();

      try {
        const relayUpdate = RelaySettingsBuilder.normal()
          .location.fromRaw(relayLocation)
          .build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.connectTunnel();
      } catch (e) {
        log.error(`Failed to select the exit location: ${e.message}`);
      }
    },
    onSelectBridgeLocation: async (bridgeLocation: RelayLocation) => {
      // dismiss the view first
      history.goBack();

      try {
        await props.app.updateBridgeLocation(bridgeLocation);
      } catch (e) {
        log.error(`Failed to select the bridge location: ${e.message}`);
      }
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(SelectLocation);
