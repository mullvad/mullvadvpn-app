import { RelaySettingsRedux } from '../redux/settings/reducers';
import RelaySettingsBuilder from '../../shared/relay-settings-builder';

export function createWireguardRelayUpdater(
  relaySettings: RelaySettingsRedux,
): ReturnType<typeof RelaySettingsBuilder['normal']> {
  if ('normal' in relaySettings) {
    const constraints = relaySettings.normal.wireguard;

    const relayUpdate = RelaySettingsBuilder.normal().tunnel.wireguard((wireguard) => {
      if (constraints.port === 'any') {
        wireguard.port.any();
      } else {
        wireguard.port.exact(constraints.port);
      }

      if (constraints.ipVersion === 'any') {
        wireguard.ipVersion.any();
      } else {
        wireguard.ipVersion.exact(constraints.ipVersion);
      }

      wireguard.useMultihop(constraints.useMultihop);

      if (constraints.entryLocation === 'any') {
        wireguard.entryLocation.any();
      } else if (constraints.entryLocation !== undefined) {
        wireguard.entryLocation.exact(constraints.entryLocation);
      }
    });

    return relayUpdate;
  } else {
    return RelaySettingsBuilder.normal();
  }
}
