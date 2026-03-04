import React from 'react';

import { useCustomLists } from '../../custom-lists/hooks';
import {
  type CustomListLocation,
  DisabledReason,
  type GeographicalLocation,
  type LocationType,
} from '../types';
import { isCustomListDisabled, isLocationSelected, searchMatchesLocation } from '../utils';
import { createLocationMap } from '../utils';
import { useCountryLocations } from './use-country-locations';
import { useSelectedLocation } from './use-selected-location';

export function useCustomListLocations(
  locationType: LocationType,
  searchTerm: string,
): CustomListLocation[] {
  const { customLists } = useCustomLists();
  const { filteredLocations, searchedLocations } = useCountryLocations(locationType, searchTerm);
  const selectedLocation = useSelectedLocation(locationType);

  const activeSearch = searchTerm.length > 0;

  const searchedLocationMap = React.useMemo(
    () => createLocationMap(searchedLocations),
    [searchedLocations],
  );
  const filteredLocationMap = React.useMemo(
    () => createLocationMap(filteredLocations),
    [filteredLocations],
  );

  const customListLocations: CustomListLocation[] = customLists.map((customList) => {
    const customListMatchesSearch = searchMatchesLocation(customList.name, searchTerm);

    const locationMap =
      activeSearch && !customListMatchesSearch ? searchedLocationMap : filteredLocationMap;

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

    const customListGeographicalLocations: GeographicalLocation[] = [];
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
    }

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
      // If not custom list matches search, one of the children did
      expanded: !customListMatchesSearch,
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
