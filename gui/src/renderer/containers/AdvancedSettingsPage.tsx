import { goBack, push } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { BridgeState, RelayProtocol, TunnelProtocol } from '../../shared/daemon-rpc-types';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import AdvancedSettings from '../components/AdvancedSettings';

import withAppContext, { IAppContext } from '../context';
import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const protocolAndPort = mapRelaySettingsToProtocolAndPort(state.settings.relaySettings);

  return {
    enableIpv6: state.settings.enableIpv6,
    blockWhenDisconnected: state.settings.blockWhenDisconnected,
    mssfix: state.settings.openVpn.mssfix,
    bridgeState: state.settings.bridgeState,
    ...protocolAndPort,
  };
};

const mapRelaySettingsToProtocolAndPort = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const { tunnelProtocol, openvpn, wireguard } = relaySettings.normal;
    return {
      openvpn: {
        protocol: openvpn.protocol === 'any' ? undefined : openvpn.protocol,
        port: openvpn.port === 'any' ? undefined : openvpn.port,
      },
      wireguard: { port: wireguard.port === 'any' ? undefined : wireguard.port },
      tunnelProtocol: tunnelProtocol === 'any' ? undefined : tunnelProtocol,
    };
    // since the GUI doesn't display custom settings, just display the default ones.
    // If the user sets any settings, then those will be applied.
  } else if ('customTunnelEndpoint' in relaySettings) {
    return {
      openvpn: {
        protocol: undefined,
        port: undefined,
      },
      wireguard: { port: undefined },
      tunnelProtocol: undefined,
    };
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const history = bindActionCreators({ push, goBack }, dispatch);
  return {
    onClose: () => {
      history.goBack();
    },
    setOpenVpnRelayProtocolAndPort: async (protocol?: RelayProtocol, port?: number) => {
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

    setWireguardRelayPort: async (port?: number) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .tunnel.wireguard((wireguard) => {
          if (port) {
            wireguard.port.exact(port);
          } else {
            wireguard.port.any();
          }
        })
        .build();
      try {
        await props.app.updateRelaySettings(relayUpdate);
      } catch (e) {
        log.error('Failed to update relay settings', e.message);
      }
    },

    setTunnelProtocol: async (tunnelProtocol: TunnelProtocol | undefined) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .tunnel.tunnelProtocol((config) => {
          if (tunnelProtocol) {
            config.tunnelProtocol.exact(tunnelProtocol);
          } else {
            config.tunnelProtocol.any();
          }
        })
        .build();
      try {
        await props.app.updateRelaySettings(relayUpdate);
      } catch (e) {
        log.error('Failed to update tunnel protocol constraints', e.message);
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

    setBridgeState: async (bridgeState: BridgeState) => {
      try {
        await props.app.setBridgeState(bridgeState);
      } catch (e) {
        log.error(`Failed to update bridge state: ${e.message}`);
      }
    },

    setOpenVpnMssfix: async (mssfix?: number) => {
      try {
        await props.app.setOpenVpnMssfix(mssfix);
      } catch (e) {
        log.error('Failed to update mssfix value', e.message);
      }
    },
    onViewWireguardKeys: () => history.push('/settings/advanced/wireguard-keys'),
  };
};

export default withAppContext(
  connect(
    mapStateToProps,
    mapDispatchToProps,
  )(AdvancedSettings),
);
