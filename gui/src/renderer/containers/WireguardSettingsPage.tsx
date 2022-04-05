import { connect } from 'react-redux';

import { IpVersion } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import WireguardSettings from '../components/WireguardSettings';
import withAppContext, { IAppContext } from '../context';
import { createWireguardRelayUpdater } from '../lib/constraint-updater';
import { IHistoryProps, withHistory } from '../lib/history';
import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, props: IAppContext) => {
  const protocolAndPort = mapRelaySettingsToProtocolAndPort(state.settings.relaySettings);

  let wireguardMultihop = false;
  if ('normal' in state.settings.relaySettings) {
    wireguardMultihop = state.settings.relaySettings.normal.wireguard.useMultihop;
  }

  return {
    wireguardMtu: state.settings.wireguard.mtu,
    wireguardMultihop,
    ...protocolAndPort,

    setWireguardPort: async (port?: number) => {
      const relayUpdate = createWireguardRelayUpdater(state.settings.relaySettings)
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

    setWireguardIpVersion: async (ipVersion?: IpVersion) => {
      const relayUpdate = createWireguardRelayUpdater(state.settings.relaySettings)
        .tunnel.wireguard((wireguard) => {
          if (ipVersion) {
            wireguard.ipVersion.exact(ipVersion);
          } else {
            wireguard.ipVersion.any();
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

    setWireguardMultihop: async (enabled: boolean) => {
      const relayUpdate = createWireguardRelayUpdater(state.settings.relaySettings)
        .tunnel.wireguard((wireguard) => wireguard.useMultihop(enabled))
        .build();
      try {
        await props.app.updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
  };
};

const mapRelaySettingsToProtocolAndPort = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const port = relaySettings.normal.wireguard.port;
    const ipVersion = relaySettings.normal.wireguard.ipVersion;
    return {
      wireguard: {
        port: port === 'any' ? undefined : port,
        ipVersion: ipVersion === 'any' ? undefined : ipVersion,
      },
    };
    // since the GUI doesn't display custom settings, just display the default ones.
    // If the user sets any settings, then those will be applied.
  } else if ('customTunnelEndpoint' in relaySettings) {
    return {
      wireguard: { port: undefined },
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

    setWireguardMtu: async (mtu?: number) => {
      try {
        await props.app.setWireguardMtu(mtu);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mtu value', error.message);
      }
    },
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(WireguardSettings)),
);
