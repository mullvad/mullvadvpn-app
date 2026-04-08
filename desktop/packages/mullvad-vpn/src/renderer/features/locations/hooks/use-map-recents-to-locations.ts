import type {
  Recents,
  RelayLocation as DaemonRelayLocation,
} from '../../../../shared/daemon-rpc-types';
import {
  type AnyLocation,
  type CityLocation,
  type CountryLocation,
  type CustomListLocation,
  type RecentLocation,
  type RecentMultihopLocation,
  type RecentSinglehopLocation,
  type RelayLocation,
} from '../types';
import { useRecents } from './use-recents';

export function useMapRecentsToLocations(
  countryLocations: CountryLocation[],
  customListLocations: CustomListLocation[],
): RecentLocation[] | undefined {
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
): RecentLocation[] {
  const findMatchingLocation = getFindMatchingLocation(
    relayLocations,
    cityLocations,
    countryLocations,
    customListLocations,
  );

  return recents
    .map((recent) => {
      if (recent.type === 'multihop') {
        const { entry, exit } = recent;
        const entryLocation = findMatchingLocation(entry);
        const exitLocation = findMatchingLocation(exit);
        if (entryLocation && exitLocation) {
          const multihopLocation: RecentMultihopLocation = {
            type: 'multihop',
            entry: entryLocation,
            exit: exitLocation,
          };
          return multihopLocation;
        }
      } else if (recent.type === 'singlehop') {
        const recentLocation = findMatchingLocation(recent.location);
        if (recentLocation) {
          const singlehopLocation: RecentSinglehopLocation = {
            type: 'singlehop',
            location: recentLocation,
          };

          return singlehopLocation;
        }
      }

      return undefined;
    })
    .filter((location) => location !== undefined);
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
