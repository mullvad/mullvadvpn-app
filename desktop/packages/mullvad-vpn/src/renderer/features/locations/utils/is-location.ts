import type {
  RelayLocation,
  RelayLocationCity,
  RelayLocationCountry,
  RelayLocationCustomList,
  RelayLocationRelay,
} from '../../../../shared/daemon-rpc-types';

export function isCountry(location: RelayLocation): location is RelayLocationCountry {
  return 'country' in location && !('city' in location);
}

export function isCity(location: RelayLocation): location is RelayLocationCity {
  return 'city' in location && !('hostname' in location);
}

export function isRelay(location: RelayLocation): location is RelayLocationRelay {
  return 'hostname' in location;
}

export function isCustomList(location: RelayLocation): location is RelayLocationCustomList {
  return 'customList' in location && !('country' in location);
}
