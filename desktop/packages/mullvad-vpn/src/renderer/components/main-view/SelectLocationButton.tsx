import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { ICustomList } from '../../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { Button, ButtonProps, Icon } from '../../lib/components';
import { transitions, useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';
import { IRelayLocationCountryRedux, RelaySettingsRedux } from '../../redux/settings/reducers';
import { useSelector } from '../../redux/store';
import { MultiButton } from '../MultiButton';

export default function SelectLocationButtons() {
  const tunnelState = useSelector((state) => state.connection.status.state);

  if (tunnelState === 'connecting' || tunnelState === 'connected') {
    return <MultiButton mainButton={SelectLocationButton} sideButton={ReconnectButton} />;
  } else {
    return <SelectLocationButton />;
  }
}

function SelectLocationButton(props: ButtonProps) {
  const { push } = useHistory();

  const tunnelState = useSelector((state) => state.connection.status.state);
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relayLocations = useSelector((state) => state.settings.relayLocations);
  const customLists = useSelector((state) => state.settings.customLists);

  const selectedRelayName = useMemo(
    () => getRelayName(relaySettings, customLists, relayLocations),
    [relaySettings, customLists, relayLocations],
  );

  const onSelectLocation = useCallback(() => {
    push(RoutePath.selectLocation, { transition: transitions.show });
  }, [push]);

  return (
    <Button
      onClick={onSelectLocation}
      aria-label={sprintf(
        messages.pgettext('accessibility', 'Select location. Current location is %(location)s'),
        { location: selectedRelayName },
      )}
      {...props}>
      <Button.Text>
        {tunnelState === 'disconnected'
          ? selectedRelayName
          : messages.pgettext('tunnel-control', 'Switch location')}
      </Button.Text>
    </Button>
  );
}

function getRelayName(
  relaySettings: RelaySettingsRedux,
  customLists: Array<ICustomList>,
  locations: IRelayLocationCountryRedux[],
): string {
  if ('normal' in relaySettings) {
    const location = relaySettings.normal.location;

    if (location === 'any') {
      return 'Automatic';
    } else if ('customList' in location) {
      return customLists.find((list) => list.id === location.customList)?.name ?? 'Unknown';
    } else if ('hostname' in location) {
      const country = locations.find(({ code }) => code === location.country);
      if (country) {
        const city = country.cities.find(({ code }) => code === location.city);
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
              hostname: location.hostname,
            },
          );
        }
      }
    } else if ('city' in location) {
      const country = locations.find(({ code }) => code === location.country);
      if (country) {
        const city = country.cities.find(({ code }) => code === location.city);
        if (city) {
          return relayLocations.gettext(city.name);
        }
      }
    } else if ('country' in location) {
      const country = locations.find(({ code }) => code === location.country);
      if (country) {
        return relayLocations.gettext(country.name);
      }
    }

    return 'Unknown';
  } else if (relaySettings.customTunnelEndpoint) {
    return 'Custom';
  } else {
    throw new Error('Unsupported relay settings.');
  }
}

const StyledReconnectButton = styled(Button)({
  minWidth: '40px',
});

function ReconnectButton(props: ButtonProps) {
  const { reconnectTunnel } = useAppContext();

  const onReconnect = useCallback(async () => {
    try {
      await reconnectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to reconnect the tunnel: ${error.message}`);
    }
  }, [reconnectTunnel]);

  return (
    <StyledReconnectButton
      onClick={onReconnect}
      size="auto"
      aria-label={messages.gettext('Reconnect')}
      {...props}>
      <Icon icon="reconnect" />
    </StyledReconnectButton>
  );
}
