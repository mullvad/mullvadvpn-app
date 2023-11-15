import { useMemo } from 'react';

import { ICustomList, RelayLocation } from '../../../shared/daemon-rpc-types';
import { hasValue } from '../../../shared/utils';
import { searchMatch } from '../../lib/filter-locations';
import { useSelector } from '../../redux/store';
import { useDisabledLocation, useSelectedLocation } from './RelayListContext';
import { isCustomListDisabled, isExpanded, isSelected } from './select-location-helpers';
import {
  CitySpecification,
  CountrySpecification,
  CustomListSpecification,
  DisabledReason,
  GeographicalRelayList,
  RelaySpecification,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

// Hook that generates the custom lists relay list.
export function useCustomListsRelayList(
  relayList: GeographicalRelayList,
  expandedLocations?: Array<RelayLocation>,
) {
  const disabledLocation = useDisabledLocation();
  const selectedLocation = useSelectedLocation();
  const { searchTerm } = useSelectLocationContext();
  const customLists = useSelector((state) => state.settings.customLists);

  // Populate all custom lists with the real location trees for the list locations.
  return useMemo(
    () =>
      customLists.map((list) =>
        prepareCustomList(
          list,
          relayList,
          searchTerm,
          selectedLocation,
          disabledLocation,
          expandedLocations,
        ),
      ),
    [customLists, relayList, selectedLocation, disabledLocation, expandedLocations],
  );
}

// Creates a CustomListSpecification from a ICustomList.
function prepareCustomList(
  list: ICustomList,
  fullRelayList: GeographicalRelayList,
  searchTerm: string,
  selectedLocation?: RelayLocation,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
  expandedLocations?: Array<RelayLocation>,
): CustomListSpecification {
  const location = { customList: list.id };
  const locations = prepareLocations(list, fullRelayList, expandedLocations);

  const disabledReason = isCustomListDisabled(location, locations, disabledLocation);
  return {
    label: list.name,
    list,
    location,
    active: disabledReason !== DisabledReason.inactive,
    disabled: disabledReason !== undefined,
    disabledReason,
    expanded: isExpanded(location, expandedLocations),
    selected: isSelected(location, selectedLocation),
    visible: searchMatch(searchTerm, list.name),
    locations,
  };
}

// Returns a list of CountrySpecification, CitySpecification and RelaySpecification matching the
// contents of the custom list.
function prepareLocations(
  list: ICustomList,
  fullRelayList: GeographicalRelayList,
  expandedLocations?: Array<RelayLocation>,
) {
  const locationCounter = {};

  return list.locations
    .map((location) => {
      if ('hostname' in location) {
        // Search through all relays in all cities in all countries to find the matching relay.
        const relay = fullRelayList
          .find((country) => country.location.country === location.country)
          ?.cities.find((city) => city.location.city === location.city)
          ?.relays.find((relay) => relay.location.hostname === location.hostname);

        return relay && updateRelay(relay, list.id);
      } else if ('city' in location) {
        // Search through all cities in all countries to find the matching city.
        const city = fullRelayList
          .find((country) => country.location.country === location.country)
          ?.cities.find((city) => city.location.city === location.city);

        return city && updateCity(city, list.id, locationCounter, expandedLocations);
      } else {
        // Search through all countries to find the matching country.
        const country = fullRelayList.find(
          (country) => country.location.country === location.country,
        );

        return country && updateCountry(country, list.id, locationCounter, expandedLocations);
      }
    })
    .filter(hasValue);
}

// Update the CountrySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCountry(
  country: CountrySpecification,
  customList: string,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<RelayLocation>,
): CountrySpecification {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${country.location.country}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...country.location, customList, count };
  return {
    ...country,
    location,
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    visible: true,
    cities: country.cities.map((city) =>
      updateCity(city, customList, locationCounter, expandedLocations),
    ),
  };
}

// Update the CitySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCity(
  city: CitySpecification,
  customList: string,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<RelayLocation>,
): CitySpecification {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${city.location.country}_${city.location.city}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...city.location, customList, count };
  return {
    ...city,
    location,
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    visible: true,
    relays: city.relays.map((relay) => updateRelay(relay, customList)),
  };
}

// Update the RelaySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateRelay(relay: RelaySpecification, customList: string): RelaySpecification {
  return {
    ...relay,
    location: { ...relay.location, customList },
    selected: false,
    visible: true,
  };
}
