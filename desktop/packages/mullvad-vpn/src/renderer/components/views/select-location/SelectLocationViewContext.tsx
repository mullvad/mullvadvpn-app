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
  setIsolatedItem: (value: LocationSelectorSelectedItem | undefined) => void;
  isLocationSelectorExpanded: boolean;
  setIsLocationSelectorExpanded: (value: boolean) => void;
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

  const [isolatedItem, stateSetIsolatedItem] = React.useState<
    LocationSelectorSelectedItem | undefined
  >(undefined);
  const setIsolatedItem = React.useCallback((value: LocationSelectorSelectedItem | undefined) => {
    React.startTransition(() => {
      stateSetIsolatedItem(value);
    });
  }, []);

  const [searchTerm, stateSetSearchTerm] = React.useState('');
  const setSearchTerm = React.useCallback((value: string) => {
    React.startTransition(() => {
      stateSetSearchTerm(value);
    });
  }, []);

  const [isLocationSelectorExpanded, setIsLocationSelectorExpanded] = React.useState(true);

  const locationType = React.useMemo(() => {
    const allowEntryLocations = multihop === 'always';
    if (allowEntryLocations) {
      return locationTypeSelector;
    }

    return LocationType.exit;
  }, [locationTypeSelector, multihop]);

  const setLocationType = React.useCallback(
    (value: LocationType) => {
      React.startTransition(() => {
        setSelectLocationView(value);
      });
    },
    [setSelectLocationView],
  );

  const value = React.useMemo(
    () => ({
      locationType,
      setLocationType,
      searchTerm,
      setSearchTerm,
      isolatedItem,
      setIsolatedItem,
      isLocationSelectorExpanded,
      setIsLocationSelectorExpanded,
    }),
    [
      locationType,
      setLocationType,
      searchTerm,
      setSearchTerm,
      isolatedItem,
      setIsolatedItem,
      isLocationSelectorExpanded,
    ],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
