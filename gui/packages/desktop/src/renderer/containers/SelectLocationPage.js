// @flow

import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { goBack } from 'connected-react-router';
import SelectLocation from '../components/SelectLocation';
import RelaySettingsBuilder from '../lib/relay-settings-builder';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  settings: state.settings,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => history.goBack(),
    onSelect: async (relayLocation) => {
      try {
        const relayUpdate = RelaySettingsBuilder.normal()
          .location.fromRaw(relayLocation)
          .build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.fetchRelaySettings();
        await props.app.connectTunnel();

        history.goBack();
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
