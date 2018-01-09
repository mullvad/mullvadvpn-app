// @flow

import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import { AdvancedSettings } from '../components/AdvancedSettings';
import RelaySettingsBuilder from '../lib/relay-settings-builder';
import { log } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => {
  const relaySettings = state.settings.relaySettings;
  if(relaySettings.normal) {
    const { protocol, port } = relaySettings.normal;
    return {
      protocol: protocol === 'any' ? 'Automatic' : protocol,
      port: port === 'any' ? 'Automatic' : port,
    };
  } else if(relaySettings.custom_tunnel_endpoint) {
    const { protocol, port } = relaySettings.custom_tunnel_endpoint;
    return { protocol, port };
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { backend } = props;
  return {
    onClose: () => dispatch(push('/settings')),

    onUpdate: async (protocol, port) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .tunnel.openvpn((openvpn) => {
          if(protocol === 'Automatic') {
            openvpn.protocol.any();
          } else {
            openvpn.protocol.exact(protocol.toLowerCase());
          }
          if(port === 'Automatic') {
            openvpn.port.any();
          } else {
            openvpn.port.exact(port);
          }
        }).build();

      try {
        await backend.updateRelaySettings(relayUpdate);
        await backend.fetchRelaySettings();
      } catch(e) {
        log.error('Failed to update relay settings', e.message);
      }
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(AdvancedSettings);
