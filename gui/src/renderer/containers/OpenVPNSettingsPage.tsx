import { connect } from 'react-redux';

import { BridgeState, RelayProtocol } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import OpenVPNSettings, { BridgeModeAvailability } from '../components/OpenVPNSettings';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const protocolAndPort = mapRelaySettingsToProtocolAndPort(state.settings.relaySettings);

  let bridgeModeAvailablity = BridgeModeAvailability.available;
  if (mapRelaySettingsToProtocol(state.settings.relaySettings) !== 'openvpn') {
    bridgeModeAvailablity = BridgeModeAvailability.blockedDueToTunnelProtocol;
  } else if (protocolAndPort.openvpn.protocol === 'udp') {
    bridgeModeAvailablity = BridgeModeAvailability.blockedDueToTransportProtocol;
  }

  return {
    bridgeModeAvailablity,
    mssfix: state.settings.openVpn.mssfix,
    bridgeState: state.settings.bridgeState,
    ...protocolAndPort,
  };
};

const mapRelaySettingsToProtocol = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const { tunnelProtocol } = relaySettings.normal;
    return tunnelProtocol === 'any' ? undefined : tunnelProtocol;
    // since the GUI doesn't display custom settings, just display the default ones.
    // If the user sets any settings, then those will be applied.
  } else if ('customTunnelEndpoint' in relaySettings) {
    return undefined;
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapRelaySettingsToProtocolAndPort = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const { openvpn } = relaySettings.normal;
    return {
      openvpn: {
        protocol: openvpn.protocol === 'any' ? undefined : openvpn.protocol,
        port: openvpn.port === 'any' ? undefined : openvpn.port,
      },
    };
    // since the GUI doesn't display custom settings, just display the default ones.
    // If the user sets any settings, then those will be applied.
  } else if ('customTunnelEndpoint' in relaySettings) {
    return {
      openvpn: {
        protocol: undefined,
        port: undefined,
      },
    };
  } else {
    throw new Error('Unknown type of relay settings.');
  }
};

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onClose: () => {
      props.history.pop();
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
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
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
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },

    setBridgeState: async (bridgeState: BridgeState) => {
      try {
        await props.app.setBridgeState(bridgeState);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to update bridge state: ${error.message}`);
      }
    },

    setOpenVpnMssfix: async (mssfix?: number) => {
      try {
        await props.app.setOpenVpnMssfix(mssfix);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mssfix value', error.message);
      }
    },
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(OpenVPNSettings)),
);
