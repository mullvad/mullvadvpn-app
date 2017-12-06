// @flow

import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import SelectLocation from '../components/SelectLocation';
import RelaySettingsBuilder from '../lib/relay-settings-builder';
import log from 'electron-log';

import type { ReduxDispatch } from '../redux/store';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch: ReduxDispatch, props) => {
  const { backend } = props;
  return {
    onClose: () => dispatch(push('/connect')),
    onSelect: async (relayLocation) => {
      try {
        const relayUpdate = RelaySettingsBuilder.normal().location.fromRaw(relayLocation).build();

        await backend.updateRelaySettings(relayUpdate);
        await backend.fetchRelaySettings();
        await backend.connect();

        dispatch(push('/connect'));
      } catch (e) {
        log.error('Failed to select server: ', e.message);
      }
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
