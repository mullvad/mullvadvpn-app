import { useMemo } from 'react';

import {
  compareRelayLocationGeographical,
  ICustomList,
  RelayLocation,
} from '../../../shared/daemon-rpc-types';
import { hasValue } from '../../../shared/utils';
import { searchMatch } from '../../lib/filter-locations';
import { useSelector } from '../../redux/store';
import {
  useDisabledLocation,
  usePreventDueToCustomBridgeSelected,
  useSelectedLocation,
} from './RelayListContext';
import {
  formatRowName,
  isCustomListDisabled,
  isExpanded,
  isSelected,
} from './select-location-helpers';
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

  const preventDueToCustomBridgeSelected = usePreventDueToCustomBridgeSelected();

  // Populate all custom lists with the real location trees for the list locations.
  return useMemo(
    () =>
      customLists.map((list) =>
        prepareCustomList(
          list,
          relayList,
          searchTerm,
          preventDueToCustomBridgeSelected,
          selectedLocation,
          disabledLocation,
          expandedLocations,
        ),
      ),
    [
      customLists,
      relayList,
      searchTerm,
      preventDueToCustomBridgeSelected,
      selectedLocation,
      disabledLocation,
      expandedLocations,
    ],
  );
}

// Creates a CustomListSpecification from a ICustomList.
function prepareCustomList(
  list: ICustomList,
  fullRelayList: GeographicalRelayList,
  searchTerm: string,
  preventDueToCustomBridgeSelected: boolean,
  selectedLocation?: RelayLocation,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
  expandedLocations?: Array<RelayLocation>,
): CustomListSpecification {
  const location = { customList: list.id };
  const locations = prepareLocations(list, fullRelayList, expandedLocations, disabledLocation);

  const disabledReason = isCustomListDisabled(location, locations, disabledLocation);

  return {
    label: formatRowName(list.name, location, disabledReason),
    list,
    location,
    active: disabledReason !== DisabledReason.inactive,
    disabled: disabledReason !== undefined,
    disabledReason,
    expanded: isExpanded(location, expandedLocations),
    selected: preventDueToCustomBridgeSelected ? false : isSelected(location, selectedLocation),
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
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
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

        return relay && updateRelay(relay, list, disabledLocation);
      } else if ('city' in location) {
        // Search through all cities in all countries to find the matching city.
        const city = fullRelayList
          .find((country) => country.location.country === location.country)
          ?.cities.find((city) => city.location.city === location.city);

        return city && updateCity(city, list, locationCounter, expandedLocations, disabledLocation);
      } else {
        // Search through all countries to find the matching country.
        const country = fullRelayList.find(
          (country) => country.location.country === location.country,
        );

        return (
          country &&
          updateCountry(country, list, locationCounter, expandedLocations, disabledLocation)
        );
      }
    })
    .filter(hasValue);
}

// Update the CountrySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCountry(
  country: CountrySpecification,
  list: ICustomList,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<RelayLocation>,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
): CountrySpecification {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${country.location.country}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...country.location, customList: list.id, count };
  return {
    ...country,
    location,
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    visible: true,
    cities: country.cities.map((city) =>
      updateCity(city, list, locationCounter, expandedLocations, disabledLocation),
    ),
  };
}

// Update the CitySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateCity(
  city: CitySpecification,
  list: ICustomList,
  locationCounter: Record<string, number>,
  expandedLocations?: Array<RelayLocation>,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
): CitySpecification {
  // Since there can be multiple instances of a location in a custom list, every instance needs to
  // be unique to avoid expanding all instances when expanding one.
  const counterKey = `${city.location.country}_${city.location.city}`;
  const count = locationCounter[counterKey] ?? 0;
  locationCounter[counterKey] = count + 1;

  const location = { ...city.location, customList: list.id, count };
  return {
    ...city,
    location,
    expanded: isExpanded(location, expandedLocations),
    selected: false,
    visible: true,
    relays: city.relays.map((relay) => updateRelay(relay, list, disabledLocation)),
  };
}

// Update the RelaySpecification from the original relay list to contain the correct properties
// for the custom list list.
function updateRelay(
  relay: RelaySpecification,
  list: ICustomList,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
): RelaySpecification {
  let disabledReason = relay.disabledReason;

  if (disabledLocation && disabledReason === undefined) {
    // If this relay's custom list parent is the disabled location and the
    // list consists of only a single item, then we should mark this relay
    // as disabled
    const isParentCustomListOfSingleItem =
      list.id === disabledLocation.location.customList && list.locations.length === 1;

    // If the relay is the same as the disabled location we should respect
    // that and mark the relay as disabled
    const isSameRelay = compareRelayLocationGeographical(relay.location, disabledLocation.location);

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
    location: { ...relay.location, customList: list.id },
    selected: false,
    visible: true,
  };
}
