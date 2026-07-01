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
import {
  getRecentMultihopEntryLocations,
  getRecentMultihopExitLocations,
  getRecentSinglehopLocations,
} from '../../../features/locations/utils';
import { useMultihop } from '../../../features/multihop/hooks';
import useActions from '../../../lib/actionsHook';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';

type SelectLocationViewContextProps = Omit<SelectLocationViewProviderProps, 'children'> & {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
  countryLocations: ReturnType<typeof useSearchCountryLocations>;
  customListLocations: ReturnType<typeof useSearchCustomListLocations>;
  recentSinglehopLocations: ReturnType<typeof getRecentSinglehopLocations>;
  recentMultihopEntryLocations: ReturnType<typeof getRecentMultihopEntryLocations>;
  recentMultihopExitLocations: ReturnType<typeof getRecentMultihopExitLocations>;
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

  const locationType = React.useMemo(() => {
    const allowEntryLocations = multihop !== 'never';
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

  const recentSinglehopLocations = getRecentSinglehopLocations(recentLocations);
  const recentMultihopEntryLocations = getRecentMultihopEntryLocations(recentLocations);
  const recentMultihopExitLocations = getRecentMultihopExitLocations(recentLocations);

  const value = React.useMemo(
    () => ({
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
      countryLocations: searchedCountryLocations,
      customListLocations: searchedCustomListLocations,
      recentSinglehopLocations,
      recentMultihopEntryLocations,
      recentMultihopExitLocations,
    }),
    [
      searchedCustomListLocations,
      searchedCountryLocations,
      locationType,
      searchTerm,
      setSearchTerm,
      setSelectLocationView,
      recentSinglehopLocations,
      recentMultihopEntryLocations,
      recentMultihopExitLocations,
    ],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
