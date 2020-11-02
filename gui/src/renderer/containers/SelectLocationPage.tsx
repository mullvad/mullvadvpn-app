import log from 'electron-log';
import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import { bindActionCreators } from 'redux';
import BridgeSettingsBuilder from '../../shared/bridge-settings-builder';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import SelectLocation from '../components/SelectLocation';
import withAppContext, { IAppContext } from '../context';
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
    relayLocations: state.settings.relayLocations,
    bridgeLocations: state.settings.bridgeLocations,
    locationScope,
    allowBridgeSelection,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  const userInterface = bindActionCreators(userInterfaceActions, dispatch);

  return {
    onClose: () => props.history.goBack(),
    onChangeLocationScope: (scope: LocationScope) => {
      userInterface.setLocationScope(scope);
    },
    onSelectExitLocation: async (relayLocation: RelayLocation) => {
      // dismiss the view first
      props.history.goBack();

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
      props.history.goBack();

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
      props.history.goBack();

      try {
        await props.app.updateBridgeSettings(new BridgeSettingsBuilder().location.any().build());
      } catch (e) {
        log.error(`Failed to set the bridge location to closest to exit: ${e.message}`);
      }
    },
  };
};

export default withAppContext(
  withRouter(connect(mapStateToProps, mapDispatchToProps)(SelectLocation)),
);
