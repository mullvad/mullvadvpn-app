import { connect } from 'react-redux';
import { TunnelProtocol } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';
import AdvancedSettings from '../components/AdvancedSettings';

import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const tunnelProtocol = mapRelaySettingsToProtocol(state.settings.relaySettings);

  return {
    enableIpv6: state.settings.enableIpv6,
    blockWhenDisconnected: state.settings.blockWhenDisconnected,
    wireguardKeyState: state.settings.wireguardKeyState,
    tunnelProtocol,
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

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onClose: () => {
      props.history.pop();
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

    onViewWireguardSettings: () => props.history.push(RoutePath.wireguardSettings),
    onViewOpenVpnSettings: () => props.history.push(RoutePath.openVpnSettings),
    onViewSplitTunneling: () => props.history.push(RoutePath.splitTunneling),
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(AdvancedSettings)),
);
