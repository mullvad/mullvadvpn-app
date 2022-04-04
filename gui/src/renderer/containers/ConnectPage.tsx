import { connect } from 'react-redux';
import { sprintf } from 'sprintf-js';

import { messages, relayLocations } from '../../shared/gettext';
import log from '../../shared/logging';
import Connect from '../components/Connect';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { IRelayLocationRedux, RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

function getRelayName(relaySettings: RelaySettingsRedux, locations: IRelayLocationRedux[]): string {
  if ('normal' in relaySettings) {
    const location = relaySettings.normal.location;

    if (location === 'any') {
      return 'Automatic';
    } else if ('country' in location) {
      const country = locations.find(({ code }) => code === location.country);
      if (country) {
        return relayLocations.gettext(country.name);
      }
    } else if ('city' in location) {
      const [countryCode, cityCode] = location.city;
      const country = locations.find(({ code }) => code === countryCode);
      if (country) {
        const city = country.cities.find(({ code }) => code === cityCode);
        if (city) {
          return relayLocations.gettext(city.name);
        }
      }
    } else if ('hostname' in location) {
      const [countryCode, cityCode, hostname] = location.hostname;
      const country = locations.find(({ code }) => code === countryCode);
      if (country) {
        const city = country.cities.find(({ code }) => code === cityCode);
        if (city) {
          return sprintf(
            // TRANSLATORS: The selected location label displayed on the main view, when a user selected a specific host to connect to.
            // TRANSLATORS: Example: Malmö (se-mma-001)
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(city)s - a city name
            // TRANSLATORS: %(hostname)s - a hostname
            messages.pgettext('connect-container', '%(city)s (%(hostname)s)'),
            {
              city: relayLocations.gettext(city.name),
              hostname,
            },
          );
        }
      }
    }

    return 'Unknown';
  } else if (relaySettings.customTunnelEndpoint) {
    return 'Custom';
  } else {
    throw new Error('Unsupported relay settings.');
  }
}

const mapStateToProps = (state: IReduxState) => {
  return {
    accountExpiry: state.account.expiry,
    loginState: state.account.status,
    blockWhenDisconnected: state.settings.blockWhenDisconnected,
    selectedRelayName: getRelayName(state.settings.relaySettings, state.settings.relayLocations),
    connection: state.connection,
  };
};

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onSelectLocation: () => {
      props.history.show(RoutePath.selectLocation);
    },
    onConnect: async () => {
      try {
        await props.app.connectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to connect the tunnel: ${error.message}`);
      }
    },
    onDisconnect: async () => {
      try {
        await props.app.disconnectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to disconnect the tunnel: ${error.message}`);
      }
    },
    onReconnect: async () => {
      try {
        await props.app.reconnectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to reconnect the tunnel: ${error.message}`);
      }
    },
  };
};

export default withAppContext(withHistory(connect(mapStateToProps, mapDispatchToProps)(Connect)));
