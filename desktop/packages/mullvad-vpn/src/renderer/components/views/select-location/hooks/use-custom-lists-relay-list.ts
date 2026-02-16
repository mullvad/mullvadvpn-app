import { useMemo } from 'react';

import {
  compareRelayLocationGeographical,
  ICustomList,
  RelayLocation as DaemonRelayLocation,
} from '../../../../../shared/daemon-rpc-types';
import { hasValue } from '../../../../../shared/utils';
import { useCustomLists } from '../../../../features/location/hooks';
import {
  formatRowName,
  isCustomListDisabled,
  isExpanded,
  isSelected,
} from '../select-location-helpers';
import {
  type CityLocation,
  type CountryLocation,
  type CustomListLocation,
  DisabledReason,
  type GeographicalLocation,
  type RelayLocation,
} from '../select-location-types';
import { useDisabledLocation } from './use-disabled-location';
import { useSelectedLocation } from './use-selected-location';

export function useCustomListsRelayList(
  relayList: CountryLocation[],
  expandedLocations?: DaemonRelayLocation[],
) {
  const disabledLocation = useDisabledLocation();
  const selectedLocation = useSelectedLocation();
  const { customLists } = useCustomLists();

  // Populate all custom lists with the real location trees for the list locations.
  return useMemo(
    () =>
      customLists.map((list) => {
        return prepareCustomList(
          list,
          relayList,
          selectedLocation,
          disabledLocation,
          expandedLocations,
        );
      }),
    [customLists, relayList, selectedLocation, disabledLocation, expandedLocations],
  );
}

// Creates a CustomListSpecification from a ICustomList.
function prepareCustomList(
  list: ICustomList,
  fullRelayList: CountryLocation[],
  selectedLocation?: DaemonRelayLocation,
  disabledLocation?: { location: DaemonRelayLocation; reason: DisabledReason },
  expandedLocations?: DaemonRelayLocation[],
): CustomListLocation {
  const location = { customList: list.id };
  const locations = prepareLocations(list, fullRelayList, expandedLocations, disabledLocation);

  const disabledReason = isCustomListDisabled(location, locations, disabledLocation);

  return {
    type: 'customList',
    details: {
      customList: list.id,
    },
    label: formatRowName(list.name, location, disabledReason),
    active: disabledReason !== DisabledReason.inactive,
    disabled: disabledReason !== undefined,
    disabledReason,
    expanded: isExpanded(location, expandedLocations),
    selected: isSelected(location, selectedLocation),
    locations,
  };
}

// Returns a list of GeographicalLocations matching the contents of the custom list.
function prepareLocations(
  customList: ICustomList,
  locationList: CountryLocation[],
  expandedLocations?: Array<DaemonRelayLocation>,
  disabledLocation?: { location: DaemonRelayLocation; reason: DisabledReason },
): GeographicalLocation[] {
  const locationCounter = {};

  return customList.locations
    .map((location) => {
      if ('hostname' in location) {
        // Search through all relays in all cities in all countries to find the matching relay.
        const relay = locationList
          .find((country) => country.details.country === location.country)
          ?.cities.find((city) => city.details.city === location.city)
          ?.relays.find((relay) => relay.details.hostname === location.hostname);

        return relay && updateRelay(relay, customList, disabledLocation);
      } else if ('city' in location) {
        // Search through all cities in all countries to find the matching city.
        const city = locationList
          .find((country) => country.details.country === location.country)
          ?.cities.find((city) => city.details.city === location.city);

        return (
          city && updateCity(city, customList, locationCounter, expandedLocations, disabledLocation)
        );
      } else {
        // Search through all countries to find the matching country.
        const country = locationList.find(
          (country) => country.details.country === location.country,
        );

        return (
          country &&
          updateCountry(country, customList, locationCounter, expandedLocations, disabledLocation)
        );
      }
    })
    .filter(hasValue);
}

// Update the CountrySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCountry(
  country: CountryLocation,
  list: ICustomList,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<DaemonRelayLocation>,
  disabledLocation?: { location: DaemonRelayLocation; reason: DisabledReason },
): CountryLocation {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${country.details.country}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...country, customList: list.id, count };
  return {
    ...country,
    type: 'country',
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    details: { ...country.details, customList: list.id },
    cities: country.cities.map((city) =>
      updateCity(city, list, locationCounter, expandedLocations, disabledLocation),
    ),
  };
}

// Update the CitySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCity(
  city: CityLocation,
  list: ICustomList,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<DaemonRelayLocation>,
  disabledLocation?: { location: DaemonRelayLocation; reason: DisabledReason },
): CityLocation {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${city.details.country}_${city.details.city}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...city, customList: list.id, count };
  return {
    ...city,
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    details: { ...city.details, customList: list.id },
    relays: city.relays.map((relay) => updateRelay(relay, list, disabledLocation)),
  };
}

// Update the RelaySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateRelay(
  relay: RelayLocation,
  list: ICustomList,
  disabledLocation?: { location: DaemonRelayLocation; reason: DisabledReason },
): RelayLocation {
  let { disabledReason } = relay;

  if (disabledLocation && disabledReason === undefined) {
    // If this relay's custom list parent is the disabled location and the
    // list consists of only a single item, then we should mark this relay
    // as disabled
    const isParentCustomListOfSingleItem =
      list.id === disabledLocation.location.customList && list.locations.length === 1;

    // If the relay is the same as the disabled location we should respect
    // that and mark the relay as disabled
    const isSameRelay = compareRelayLocationGeographical(relay.details, disabledLocation.location);

    if (isParentCustomListOfSingleItem || isSameRelay) {
      if (
        disabledLocation.reason === DisabledReason.exit ||
        disabledLocation.reason === DisabledReason.entry
      ) {
        disabledReason = disabledLocation.reason;
      }
    }
  }

  return {
    ...relay,
    disabledReason,
    disabled: relay.disabled || disabledReason !== undefined,
    details: { ...relay.details, customList: list.id },
    selected: false,
  };
}
