import type { RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useCustomLists } from '../../../../../../features/custom-lists/hooks';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import {
  findCity,
  findCountry,
  findCustomList,
  findRelay,
  isCity,
  isCountry,
  isCustomList,
  isRelay,
} from '../../../../../../features/locations/utils';

export function useLocationName(location: RelayLocation | undefined): string | undefined {
  const { relayLocations } = useRelayLocations();
  const { customLists } = useCustomLists();

  if (!location) {
    return undefined;
  }

  if (isCustomList(location)) {
    const customList = findCustomList(location.customList, customLists);
    return customList?.name;
  }
  if (isRelay(location)) {
    const relay = findRelay(location.hostname, relayLocations);
    return relay?.hostname;
  } else if (isCity(location)) {
    const city = findCity(location.city, relayLocations);
    return city?.name;
  } else if (isCountry(location)) {
    const country = findCountry(location.country, relayLocations);
    return country?.name;
  }

  return undefined;
}
