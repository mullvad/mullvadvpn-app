import { RelayLocation } from '../../../../shared/daemon-rpc-types';
import { isCity, isCountry, isCustomList, isRelay } from './is-location';

export function isSameLocation(
  locationOne: RelayLocation | undefined,
  locationTwo: RelayLocation | undefined,
) {
  if (locationOne && locationTwo) {
    if (isCustomList(locationOne) && isCustomList(locationTwo)) {
      return locationOne.customList == locationTwo.customList;
    }

    if (isRelay(locationOne) && isRelay(locationTwo)) {
      return locationOne.hostname === locationTwo.hostname;
    }

    if (isCity(locationOne) && isCity(locationTwo)) {
      return locationOne.city == locationTwo.city;
    }

    if (isCountry(locationOne) && isCountry(locationTwo)) {
      return locationOne.country === locationTwo.country;
    }
  }

  return false;
}
