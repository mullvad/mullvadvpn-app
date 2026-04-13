import React from 'react';

import {
  useFilterCountryLocations,
  useMapCustomListsToLocations,
  useMapReduxCountriesToCountryLocations,
  useSearchCountryLocations,
  useSearchCustomListLocations,
  useSelectedLocation,
} from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import useActions from '../../../lib/actionsHook';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';

type SelectLocationViewContextProps = Omit<SelectLocationViewProviderProps, 'children'> & {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
  countryLocations: ReturnType<typeof useSearchCountryLocations>;
  customListLocations: ReturnType<typeof useSearchCustomListLocations>;
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
  const relaySettings = useNormalRelaySettings();
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);

  const locationType = React.useMemo(() => {
    const allowEntryLocations = relaySettings?.wireguard.useMultihop;

    if (allowEntryLocations) {
      return locationTypeSelector;
    }
    return LocationType.exit;
  }, [locationTypeSelector, relaySettings]);

  const filteredCountries = useFilterCountryLocations(locationType);
  const filteredCountryLocations = useMapReduxCountriesToCountryLocations(
    locationType,
    filteredCountries,
  );
  const searchedCountryLocations = useSearchCountryLocations(filteredCountryLocations, searchTerm);

  const selectedLocation = useSelectedLocation(locationType);

  const filteredCustomListLocations = useMapCustomListsToLocations(
    searchedCountryLocations,
    searchTerm,
    selectedLocation,
  );
  const searchedCustomListLocations = useSearchCustomListLocations(
    filteredCustomListLocations,
    searchTerm,
  );

  const value = React.useMemo(
    () => ({
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
      countryLocations: searchedCountryLocations,
      customListLocations: searchedCustomListLocations,
    }),
    [
      searchedCustomListLocations,
      searchedCountryLocations,
      locationType,
      searchTerm,
      setSearchTerm,
      setSelectLocationView,
    ],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
