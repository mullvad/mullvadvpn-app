import React from 'react';

import { LocationType } from '../../../features/locations/types';
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
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);
  const { multihop } = useMultihop();
  const [searchTerm, setSearchTerm] = React.useState('');
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

  const value = React.useMemo(
    () => ({
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
      isolatedItem,
      setIsolatedItem,
    }),
    [locationType, setSelectLocationView, searchTerm, isolatedItem],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
