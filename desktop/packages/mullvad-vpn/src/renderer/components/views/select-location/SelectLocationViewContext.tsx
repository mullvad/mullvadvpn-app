import React from 'react';

import {
  useCustomListLocations,
  useFilteredCountryLocations,
  useSearchCountryLocations,
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
  filteredLocations: ReturnType<typeof useFilteredCountryLocations>;
  searchedLocations: ReturnType<typeof useSearchCountryLocations>;
  customListLocations: ReturnType<typeof useCustomListLocations>;
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
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);
  const { setSelectLocationView } = useActions(userInterface);
  const [searchTerm, setSearchTerm] = React.useState('');
  const relaySettings = useNormalRelaySettings();
  const selectedLocation = useSelectedLocation(locationTypeSelector);
  const filteredLocations = useFilteredCountryLocations(locationTypeSelector);
  const searchedLocations = useSearchCountryLocations(filteredLocations, searchTerm);

  const activeSearch = searchTerm.length > 0;

  const customListLocations = useCustomListLocations({
    locations: activeSearch ? searchedLocations : filteredLocations,
    selectedLocation,
    searchTerm,
  });

  const locationType = React.useMemo(() => {
    const allowEntryLocations = relaySettings?.wireguard.useMultihop;

    if (allowEntryLocations) {
      return locationTypeSelector;
    }
    return LocationType.exit;
  }, [locationTypeSelector, relaySettings]);

  const value = React.useMemo(
    () => ({
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
      filteredLocations,
      searchedLocations,
      customListLocations,
    }),
    [
      customListLocations,
      filteredLocations,
      locationType,
      searchTerm,
      searchedLocations,
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
