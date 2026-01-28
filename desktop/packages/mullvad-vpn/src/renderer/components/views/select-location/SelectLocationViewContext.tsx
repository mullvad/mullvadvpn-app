import React from 'react';

import useActions from '../../../lib/actionsHook';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';
import { LocationType } from './select-location-types';

type SelectLocationViewContextProps = {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
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
    }),
    [locationType, searchTerm, setSearchTerm, setSelectLocationView],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
