import {
  compareRelayLocation,
  type RelayLocationCountry,
} from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { useDisabledLocation } from '../hooks';
import { DisabledReason } from '../types';
import { isCityDisabled } from './is-city-disabled';

export function isCountryDisabled(
  country: IRelayLocationCountryRedux,
  location: RelayLocationCountry,
  disabledLocation?: ReturnType<typeof useDisabledLocation>,
): DisabledReason | undefined {
  const citiesDisabled = country.cities.map((city) =>
    isCityDisabled(city, { ...location, city: city.code }, disabledLocation),
  );
  if (citiesDisabled.every((status) => status === DisabledReason.inactive)) {
    return DisabledReason.inactive;
  }

  const disabledDueToSelection = citiesDisabled.find(
    (status) => status === DisabledReason.entry || status === DisabledReason.exit,
  );
  if (
    citiesDisabled.every((status) => status !== undefined) &&
    disabledDueToSelection !== undefined
  ) {
    return disabledDueToSelection;
  }

  if (
    disabledLocation?.location &&
    compareRelayLocation(location, disabledLocation.location) &&
    country.cities.flatMap((city) => city.relays).filter((relay) => relay.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}
