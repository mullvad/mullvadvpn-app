// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import SelectLocation from '../components/SelectLocation';
import RelaySettingsBuilder from '../lib/relay-settings-builder';
import { log } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  settings: state.settings,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    onClose: () => pushHistory('/connect'),
    onSelect: async (relayLocation) => {
      try {
        const relayUpdate = RelaySettingsBuilder.normal()
          .location.fromRaw(relayLocation)
          .build();

        await props.app.updateRelaySettings(relayUpdate);
        await props.app.fetchRelaySettings();
        await props.app.connectTunnel();

        pushHistory('/connect');
      } catch (e) {
        log.error('Failed to select server: ', e.message);
      }
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(SelectLocation);
