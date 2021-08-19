import { connect } from 'react-redux';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import WireguardSettings from '../components/WireguardSettings';

import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const protocolAndPort = mapRelaySettingsToProtocolAndPort(state.settings.relaySettings);

  return {
    wireguardMtu: state.settings.wireguard.mtu,
    ...protocolAndPort,
  };
};

const mapRelaySettingsToProtocolAndPort = (relaySettings: RelaySettingsRedux) => {
  if ('normal' in relaySettings) {
    const port = relaySettings.normal.wireguard.port;
    return { wireguard: { port: port === 'any' ? undefined : port } };
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

    setWireguardMtu: async (mtu?: number) => {
      try {
        await props.app.setWireguardMtu(mtu);
      } catch (e) {
        log.error('Failed to update mtu value', e.message);
      }
    },

    onViewWireguardKeys: () => props.history.push(RoutePath.wireguardKeys),
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(WireguardSettings)),
);
