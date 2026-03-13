import type { RelayLocation as DaemonRelayLocation } from '../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import type { useDisabledLocation, useSelectedLocation } from '../hooks';
import { DisabledReason, type RelayLocation } from '../types';
import { createLocationLabel } from './create-location-label';
import { isLocationSelected } from './is-location-selected';
import { isRelayDisabled } from './is-relay-disabled';

export function mapReduxRelayToRelayLocation(
  country: IRelayLocationCountryRedux,
  city: IRelayLocationCityRedux,
  relay: IRelayLocationRelayRedux,
  selectedLocation: ReturnType<typeof useSelectedLocation>,
  disabledLocation: ReturnType<typeof useDisabledLocation>,
  parentDisabledReason: DisabledReason | undefined,
): RelayLocation {
  const relayLocation: DaemonRelayLocation = {
    country: country.code,
    city: city.code,
    hostname: relay.hostname,
  };

  const relayDisabledReason =
    parentDisabledReason ?? isRelayDisabled(relay, relayLocation, disabledLocation);
  const label = createLocationLabel(relay.hostname, relayLocation, relayDisabledReason);

  return {
    type: 'relay',
    label,
    details: {
      country: country.code,
      city: city.code,
      hostname: relay.hostname,
    },
    active: relayDisabledReason !== DisabledReason.inactive,
    disabled: relayDisabledReason !== undefined,
    disabledReason: relayDisabledReason,
    selected: isLocationSelected(relayLocation, selectedLocation),
    expanded: false,
  };
}
