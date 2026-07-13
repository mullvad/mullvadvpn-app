import React from 'react';

import { LocationType } from '../../../features/locations/types';
import { useMultihop } from '../../../features/multihop/hooks';
import useActions from '../../../lib/actionsHook';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';

type SelectLocationViewContextProps = Omit<SelectLocationViewProviderProps, 'children'> & {
  entryLocationListsContainerRef: React.RefObject<HTMLDivElement | null>;
  exitLocationListsContainerRef: React.RefObject<HTMLDivElement | null>;
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
  const { setSelectLocationView } = useActions(userInterface);
  const [searchTerm, setSearchTerm] = React.useState('');
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);
  const { multihop } = useMultihop();
  const entryLocationListsContainerRef = React.useRef<HTMLDivElement | null>(null);
  const exitLocationListsContainerRef = React.useRef<HTMLDivElement | null>(null);

  const locationType = React.useMemo(() => {
    const allowEntryLocations = multihop !== 'never';
    if (allowEntryLocations) {
      return locationTypeSelector;
    }

    return LocationType.exit;
  }, [locationTypeSelector, multihop]);

  const value = React.useMemo(
    () => ({
      entryLocationListsContainerRef,
      exitLocationListsContainerRef,
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
