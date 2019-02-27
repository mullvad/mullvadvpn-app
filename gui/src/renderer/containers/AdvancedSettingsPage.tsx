import { goBack } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { RelayProtocol } from '../../shared/daemon-rpc-types';
import AdvancedSettings from '../components/AdvancedSettings';
import RelaySettingsBuilder from '../lib/relay-settings-builder';

import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => {
  const protocolAndPort = mapRelaySettingsToProtocolAndPort(state.settings.relaySettings);

  return {
    enableIpv6: state.settings.enableIpv6,
    blockWhenDisconnected: state.settings.blockWhenDisconnected,
    mssfix: state.settings.openVpn.mssfix,
    ...protocolAndPort,
  };
};

const mapRelaySettingsToProtocolAndPort = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const { protocol, port } = relaySettings.normal;
    return {
      protocol: protocol === 'any' ? undefined : protocol,
      port: port === 'any' ? undefined : port,
    };
  } else if ('customTunnelEndpoint' in relaySettings) {
    const { protocol, port } = relaySettings.customTunnelEndpoint;
    return { protocol, port };
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => {
      history.goBack();
    },
    setRelayProtocolAndPort: async (protocol?: RelayProtocol, port?: number) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .tunnel.openvpn((openvpn) => {
          if (protocol) {
            openvpn.protocol.exact(protocol);
          } else {
            openvpn.protocol.any();
          }

          if (port) {
            openvpn.port.exact(port);
          } else {
            openvpn.port.any();
          }
        })
        .build();

      try {
        await props.app.updateRelaySettings(relayUpdate);
      } catch (e) {
        log.error('Failed to update relay settings', e.message);
      }
    },

    setEnableIpv6: async (enableIpv6: boolean) => {
      try {
        await props.app.setEnableIpv6(enableIpv6);
      } catch (e) {
        log.error('Failed to update enable IPv6', e.message);
      }
    },

    setBlockWhenDisconnected: async (blockWhenDisconnected: boolean) => {
      try {
        await props.app.setBlockWhenDisconnected(blockWhenDisconnected);
      } catch (e) {
        log.error('Failed to update block when disconnected', e.message);
      }
    },

    setOpenVpnMssfix: async (mssfix?: number) => {
      try {
        await props.app.setOpenVpnMssfix(mssfix);
      } catch (e) {
        log.error('Failed to update mssfix value', e.message);
      }
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(AdvancedSettings);
