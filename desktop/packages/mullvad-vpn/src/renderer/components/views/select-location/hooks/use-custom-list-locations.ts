import React from 'react';

import { useCustomLists } from '../../../../features/location/hooks';
import { useLocationsContext } from '../LocationsContext';
import { isCustomListDisabled, isSelected } from '../select-location-helpers';
import {
  type CustomListLocation,
  DisabledReason,
  type GeographicalLocation,
} from '../select-location-types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { createLocationMap, searchMatchesLocation } from '../utils';
import { useSelectedLocation } from './use-selected-location';

export function useCustomListLocations(): CustomListLocation[] {
  const { searchTerm } = useSelectLocationViewContext();
  const { filteredLocations, searchedLocations } = useLocationsContext();
  const { customLists } = useCustomLists();
  const selectedLocation = useSelectedLocation();

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
      details,
      disabled: disabledReason !== undefined,
      disabledReason,
      locations: customListGeographicalLocations,
      active: disabledReason !== DisabledReason.inactive,
      // If not custom list matches search, one of the children did
      expanded: !customListMatchesSearch,
      selected: isSelected(details, selectedLocation),
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
