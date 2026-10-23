import type { LiftedConstraint, RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
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
import { useSettingsCustomLists } from '../../../../../../redux/hooks';
import { useAutomaticLocationName } from './use-automatic-location-name';

export function useLocationName(location: LiftedConstraint<RelayLocation>): string | undefined {
  const { relayLocations } = useRelayLocations();
  const { customLists } = useSettingsCustomLists();
  const automaticName = useAutomaticLocationName();

  if (location === 'any') {
    return automaticName;
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
