import type {
  Recents,
  RelayLocation as DaemonRelayLocation,
} from '../../../../shared/daemon-rpc-types';
import {
  type AnyLocation,
  type CityLocation,
  type CountryLocation,
  type CustomListLocation,
  LocationType,
  type RelayLocation,
} from '../types';
import { useRecents } from './use-recents';

export function useMapRecentsToLocations(
  countryLocations: CountryLocation[],
  customListLocations: CustomListLocation[],
): { entry: AnyLocation[]; exit: AnyLocation[] } {
  const { recents } = useRecents();

  const relayLocations = countryLocations.flatMap((country) =>
    country.cities.flatMap((city) => city.relays),
  );

  const cityLocations = countryLocations.flatMap((country) => country.cities);

  const recentEntries = recents.filter((recent) => recent.entry);
  const recentEntryLocations = getRecentLocations(
    LocationType.entry,
    recentEntries,
    customListLocations,
    countryLocations,
    cityLocations,
    relayLocations,
  );

  const recentExits = recents.filter((recent) => recent.exit);
  const recentExitLocations = getRecentLocations(
    LocationType.exit,
    recentExits,
    customListLocations,
    countryLocations,
    cityLocations,
    relayLocations,
  );

  return { entry: recentEntryLocations, exit: recentExitLocations };
}

function getRecentLocations(
  locationType: LocationType,
  recents: Recents,
  customListLocations: CustomListLocation[],
  countryLocations: CountryLocation[],
  cityLocations: CityLocation[],
  relayLocations: RelayLocation[],
): AnyLocation[] {
  const addedRecentLocations = new Set<string>();
  return recents
    .map((recent) => {
      const { entry, exit } = recent;
      let recentLocation: AnyLocation | undefined;
      if (locationType === LocationType.entry && entry) {
        recentLocation = findMatchingLocation(
          entry,
          relayLocations,
          cityLocations,
          countryLocations,
          customListLocations,
        );
      } else if (locationType === LocationType.exit) {
        recentLocation = findMatchingLocation(
          exit,
          relayLocations,
          cityLocations,
          countryLocations,
          customListLocations,
        );
      }

      if (recentLocation) {
        const locationKey = JSON.stringify(recentLocation.details);
        if (!addedRecentLocations.has(locationKey)) {
          addedRecentLocations.add(locationKey);
          return recentLocation;
        }
      }

      return undefined;
    })
    .filter((location) => location !== undefined)
    .slice(0, 3);
}

function findMatchingLocation(
  relayLocation: DaemonRelayLocation,
  relayLocations: RelayLocation[],
  cityLocations: CityLocation[],
  countryLocations: CountryLocation[],
  customListLocations: CustomListLocation[],
): AnyLocation | undefined {
  if ('hostname' in relayLocation) {
    return relayLocations.find((location) => location.details.hostname === relayLocation.hostname);
  }
  if ('city' in relayLocation) {
    return cityLocations.find((location) => location.details.city === relayLocation.city);
  }
  if ('country' in relayLocation) {
    return countryLocations.find((location) => location.details.country === relayLocation.country);
  }
  if ('customList' in relayLocation) {
    return customListLocations.find(
      (location) => location.details.customList === relayLocation.customList,
    );
  }

  return undefined;
}
