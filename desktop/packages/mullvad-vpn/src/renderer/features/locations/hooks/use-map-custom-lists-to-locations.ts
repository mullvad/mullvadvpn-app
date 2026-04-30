import type { RelayLocation } from '../../../../shared/daemon-rpc-types';
import { useCustomLists } from '../../custom-lists/hooks';
import {
  type CountryLocation,
  type CustomListLocation,
  DisabledReason,
  type GeographicalLocation,
} from '../types';
import {
  createCountryLocationMap,
  isCustomListDisabled,
  isLocationSelected,
  searchMatchesLocation,
} from '../utils';

export function useMapCustomListsToLocations(
  countryLocations: CountryLocation[],
  searchTerm: string,
  selectedLocation?: RelayLocation,
): CustomListLocation[] {
  const { customLists } = useCustomLists();

  const customListLocations: CustomListLocation[] = customLists.map((customList) => {
    const locationMap = createCountryLocationMap(countryLocations);

    // Get all ids of locations that are in the custom list
    const customListLocationIds = customList.locations.flatMap((location) => {
      if ('hostname' in location) {
        return location.hostname;
      }
      if ('city' in location) {
        return location.city;
      }

      return location.country;
    });

    // Pick the locations from the map that are in the custom list, and add custom list details to them
    const customListGeographicalLocations: GeographicalLocation[] = [];
    let childLocationExpandedOrSelected = false;
    for (const id of customListLocationIds) {
      const location = locationMap.get(id);
      if (!location) {
        continue;
      }

      const customListGeographicalLocation = {
        ...location,
        details: {
          ...location.details,
          customList: customList.id,
        },
      } as GeographicalLocation;

      customListGeographicalLocations.push(customListGeographicalLocation);
      if (customListGeographicalLocation.expanded || customListGeographicalLocation.selected) {
        childLocationExpandedOrSelected = true;
      }
    }

    const customListMatchesSearch = searchMatchesLocation(customList.name, searchTerm);
    // Expand if one of the child locations are expanded or selected, or if the custom list itself does
    // not match the search, since that means one of the children must have matched.
    const customListExpanded = !customListMatchesSearch || childLocationExpandedOrSelected;

    const details = {
      customList: customList.id,
    };

    const disabledReason = isCustomListDisabled(details, customListGeographicalLocations);

    return {
      type: 'customList',
      label: customList.name,
      searchText: customList.name.toLowerCase(),
      details,
      disabled: disabledReason !== undefined,
      disabledReason,
      locations: customListGeographicalLocations,
      active: disabledReason !== DisabledReason.inactive,
      expanded: customListExpanded,
      selected: isLocationSelected(details, selectedLocation),
    };
  });

  if (searchTerm.length > 0) {
    return customListLocations.filter((customList) => {
      if (searchMatchesLocation(customList.label, searchTerm)) {
        return true;
      }

      return customList.locations.length > 0;
    });
  }

  return customListLocations;
}
