import { goBack } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { RelayLocation } from '../../shared/daemon-rpc-types';
import SelectLocation from '../components/SelectLocation';
import RelaySettingsBuilder from '../lib/relay-settings-builder';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => ({
  relaySettings: state.settings.relaySettings,
  relayLocations: state.settings.relayLocations,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => history.goBack(),
    onSelect: async (relayLocation: RelayLocation) => {
      // dismiss the view first
      history.goBack();

      try {
        const relayUpdate = RelaySettingsBuilder.normal()
          .location.fromRaw(relayLocation)
          .build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.connectTunnel();
      } catch (e) {
        log.error(`Failed to select server: ${e.message}`);
      }
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(SelectLocation);
