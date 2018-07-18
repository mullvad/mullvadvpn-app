// @flow

import log from 'electron-log';
import { connect } from 'react-redux';
import { goBack } from 'connected-react-router';
import { bindActionCreators } from 'redux';
import { AdvancedSettings } from '../components/AdvancedSettings';
import RelaySettingsBuilder from '../lib/relay-settings-builder';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => {
  const relaySettings = state.settings.relaySettings;
  if (relaySettings.normal) {
    const { protocol, port } = relaySettings.normal;
    return {
      protocol: protocol === 'any' ? 'Automatic' : protocol,
      port: port === 'any' ? 'Automatic' : port,
    };
  } else if (relaySettings.custom_tunnel_endpoint) {
    const { protocol, port } = relaySettings.custom_tunnel_endpoint;
    return { protocol, port };
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => {
      history.goBack();
    },
    onUpdate: async (protocol, port) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .tunnel.openvpn((openvpn) => {
          if (protocol === 'Automatic') {
            openvpn.protocol.any();
          } else {
            openvpn.protocol.exact(protocol.toLowerCase());
          }
          if (port === 'Automatic') {
            openvpn.port.any();
          } else {
            openvpn.port.exact(port);
          }
        })
        .build();

      try {
        await props.app.updateRelaySettings(relayUpdate);
        await props.app.fetchRelaySettings();
      } catch (e) {
        log.error('Failed to update relay settings', e.message);
      }
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(AdvancedSettings);
