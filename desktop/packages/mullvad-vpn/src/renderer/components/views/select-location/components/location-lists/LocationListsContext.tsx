import React from 'react';

import {
  useFilterCountryLocations,
  useMapCustomListsToLocations,
  useMapRecentsToLocations,
  useMapReduxCountriesToCountryLocations,
  useSearchCountryLocations,
  useSearchCustomListLocations,
  useSelectedEntryOrExitLocation,
} from '../../../../../features/locations/hooks';
import { type AnyLocation, LocationType } from '../../../../../features/locations/types';
import {
  getRecentEntryLocations,
  getRecentExitLocations,
} from '../../../../../features/locations/utils';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { useHandleSelectEntryLocation, useHandleSelectExitLocation } from './hooks';
import type { LocationsListsProps } from './LocationLists';

type LocationListsContextProps = Omit<LocationListsProviderProps, 'children'> & {
  handleSelect: (location: AnyLocation) => Promise<void> | void;
  countryLocations: ReturnType<typeof useSearchCountryLocations>;
  customListLocations: ReturnType<typeof useSearchCustomListLocations>;
  recentEntryLocations: ReturnType<typeof getRecentEntryLocations>;
  recentExitLocations: ReturnType<typeof getRecentExitLocations>;
};

const LocationListsContext = React.createContext<LocationListsContextProps | undefined>(undefined);

export const useLocationListsContext = (): LocationListsContextProps => {
  const context = React.useContext(LocationListsContext);
  if (!context) {
    throw new Error('useLocationListsContext must be used within a LocationListsProvider');
  }
  return context;
};

type LocationListsProviderProps = React.PropsWithChildren & {
  type: LocationsListsProps['type'];
};

export function LocationListsProvider({ type, children }: LocationListsProviderProps) {
  const { searchTerm } = useSelectLocationViewContext();
  const handleSelectExitLocation = useHandleSelectExitLocation();
  const handleSelectEntryLocation = useHandleSelectEntryLocation();

  const handleSelect = React.useMemo(() => {
    if (type === LocationType.entry) {
      return handleSelectEntryLocation;
    }
    return handleSelectExitLocation;
  }, [type, handleSelectEntryLocation, handleSelectExitLocation]);

  const filteredCountries = useFilterCountryLocations(type);
  const filteredCountryLocations = useMapReduxCountriesToCountryLocations(type, filteredCountries);
  const searchedCountryLocations = useSearchCountryLocations(filteredCountryLocations, searchTerm);

  const selectedLocation = useSelectedEntryOrExitLocation(type);

  const filteredCustomListLocations = useMapCustomListsToLocations(
    searchedCountryLocations,
    searchTerm,
    selectedLocation,
  );
  const searchedCustomListLocations = useSearchCustomListLocations(
    filteredCustomListLocations,
    searchTerm,
  );

  const recentLocations = useMapRecentsToLocations(
    searchedCountryLocations,
    searchedCustomListLocations,
  );

  const recentEntryLocations = getRecentEntryLocations(recentLocations);
  const recentExitLocations = getRecentExitLocations(recentLocations);

  const value = React.useMemo(
    () => ({
      type,
      handleSelect,
      countryLocations: searchedCountryLocations,
      customListLocations: searchedCustomListLocations,
      recentEntryLocations,
      recentExitLocations,
    }),
    [
      type,
      handleSelect,
      searchedCountryLocations,
      searchedCustomListLocations,
      recentEntryLocations,
      recentExitLocations,
    ],
  );

  return <LocationListsContext.Provider value={value}>{children}</LocationListsContext.Provider>;
}
