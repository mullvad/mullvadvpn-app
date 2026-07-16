import React from 'react';

import {
  useFilterCountryLocations,
  useMapCustomListsToLocations,
  useMapRecentsToLocations,
  useMapReduxCountriesToCountryLocations,
  useSearchCountryLocations,
  useSearchCustomListLocations,
  useSelectedEntryOrExitLocation,
} from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { getRecentEntryLocations, getRecentExitLocations } from '../../../features/locations/utils';
import { useMultihop } from '../../../features/multihop/hooks';
import useActions from '../../../lib/actionsHook';
import type { LocationSelectorSelectedItem } from '../../../lib/components/location-selector';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';

type SelectLocationViewContextProps = Omit<SelectLocationViewProviderProps, 'children'> & {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
  countryLocations: ReturnType<typeof useSearchCountryLocations>;
  customListLocations: ReturnType<typeof useSearchCustomListLocations>;
  recentEntryLocations: ReturnType<typeof getRecentEntryLocations>;
  recentExitLocations: ReturnType<typeof getRecentExitLocations>;
  isolatedItem: LocationSelectorSelectedItem | undefined;
  setIsolatedItem: React.Dispatch<React.SetStateAction<LocationSelectorSelectedItem | undefined>>;
};

const SelectLocationViewContext = React.createContext<SelectLocationViewContextProps | undefined>(
  undefined,
);

export const useSelectLocationViewContext = (): SelectLocationViewContextProps => {
  const context = React.useContext(SelectLocationViewContext);
  if (!context) {
    throw new Error(
      'useSelectLocationViewContext must be used within a SelectLocationViewProvider',
    );
  }
  return context;
};

type SelectLocationViewProviderProps = React.PropsWithChildren;

export function SelectLocationViewProvider({ children }: SelectLocationViewProviderProps) {
  const { setSelectLocationView } = useActions(userInterface);
  const [searchTerm, setSearchTerm] = React.useState('');
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);
  const { multihop } = useMultihop();
  const [isolatedItem, setIsolatedItem] = React.useState<LocationSelectorSelectedItem | undefined>(
    undefined,
  );

  const locationType = React.useMemo(() => {
    const allowEntryLocations = multihop === 'always';
    if (allowEntryLocations) {
      return locationTypeSelector;
    }

    return LocationType.exit;
  }, [locationTypeSelector, multihop]);

  const filteredCountries = useFilterCountryLocations(locationType);
  const filteredCountryLocations = useMapReduxCountriesToCountryLocations(
    locationType,
    filteredCountries,
  );
  const searchedCountryLocations = useSearchCountryLocations(filteredCountryLocations, searchTerm);

  const selectedLocation = useSelectedEntryOrExitLocation(locationType);

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
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
      countryLocations: searchedCountryLocations,
      customListLocations: searchedCustomListLocations,
      recentEntryLocations,
      recentExitLocations,
      isolatedItem,
      setIsolatedItem,
    }),
    [
      locationType,
      setSelectLocationView,
      searchTerm,
      searchedCountryLocations,
      searchedCustomListLocations,
      recentEntryLocations,
      recentExitLocations,
      isolatedItem,
    ],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
