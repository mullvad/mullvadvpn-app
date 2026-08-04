import type {
  Recents,
  RelayLocation as DaemonRelayLocation,
} from '../../../../shared/daemon-rpc-types';
import {
  type AnyLocation,
  type CityLocation,
  type CountryLocation,
  type CustomListLocation,
  type RecentLocations,
  type RelayLocation,
} from '../types';
import { useRecents } from './use-recents';

export function useMapRecentsToLocations(
  countryLocations: CountryLocation[],
  customListLocations: CustomListLocation[],
): RecentLocations | undefined {
  const { recents } = useRecents();

  if (!recents) {
    return undefined;
  }

  const relayLocations = countryLocations.flatMap((country) =>
    country.cities.flatMap((city) => city.relays),
  );

  const cityLocations = countryLocations.flatMap((country) => country.cities);

  const recentLocations = getRecentLocations(
    recents,
    customListLocations,
    countryLocations,
    cityLocations,
    relayLocations,
  );

  return recentLocations;
}

function getRecentLocations(
  recents: Recents,
  customListLocations: CustomListLocation[],
  countryLocations: CountryLocation[],
  cityLocations: CityLocation[],
  relayLocations: RelayLocation[],
): RecentLocations {
  const findMatchingLocation = getFindMatchingLocation(
    relayLocations,
    cityLocations,
    countryLocations,
    customListLocations,
  );

  const { entries, exits } = recents;
  const recentEntryLocations = entries
    .map((entry) => findMatchingLocation(entry))
    .filter((location) => location !== undefined);
  const recentExitLocations = exits
    .map((exit) => findMatchingLocation(exit))
    .filter((location) => location !== undefined);

  return {
    entries: recentEntryLocations,
    exits: recentExitLocations,
  };
}

function getFindMatchingLocation(
  relayLocations: RelayLocation[],
  cityLocations: CityLocation[],
  countryLocations: CountryLocation[],
  customListLocations: CustomListLocation[],
): (relayLocation: DaemonRelayLocation) => AnyLocation | undefined {
  return (relayLocation: DaemonRelayLocation) => {
    if ('hostname' in relayLocation) {
      return relayLocations.find(
        (location) => location.details.hostname === relayLocation.hostname,
      );
    }
    if ('city' in relayLocation) {
      return cityLocations.find((location) => location.details.city === relayLocation.city);
    }
    if ('country' in relayLocation) {
      return countryLocations.find(
        (location) => location.details.country === relayLocation.country,
      );
    }
    if ('customList' in relayLocation) {
      return customListLocations.find(
        (location) => location.details.customList === relayLocation.customList,
      );
    }

    return undefined;
  };
}
