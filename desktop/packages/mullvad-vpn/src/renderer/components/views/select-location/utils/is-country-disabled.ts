import {
  compareRelayLocation,
  type RelayLocation,
  type RelayLocationCountry,
} from '../../../../../shared/daemon-rpc-types';
import type { IRelayLocationCountryRedux } from '../../../../redux/settings/reducers';
import { DisabledReason } from '../select-location-types';
import { isCityDisabled } from './is-city-disabled';

export function isCountryDisabled(
  country: IRelayLocationCountryRedux,
  location: RelayLocationCountry,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
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
    disabledLocation &&
    compareRelayLocation(location, disabledLocation.location) &&
    country.cities.flatMap((city) => city.relays).filter((relay) => relay.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}
