import React from 'react';

import { LocationType } from '../../../features/locations/types';
import useActions from '../../../lib/actionsHook';
import { useSelector } from '../../../redux/store';
import userInterface from '../../../redux/userinterface/actions';

type SelectLocationViewContextProps = Omit<SelectLocationViewProviderProps, 'children'> & {
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
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);

  const [searchTerm, stateSetSearchTerm] = React.useState('');
  const setSearchTerm = React.useCallback((value: string) => {
    React.startTransition(() => {
      stateSetSearchTerm(value);
    });
  }, []);

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
      locationType: locationTypeSelector,
      setLocationType,
      searchTerm,
      setSearchTerm,
    }),
    [locationTypeSelector, setLocationType, searchTerm, setSearchTerm],
  );

  return (
    <SelectLocationViewContext.Provider value={value}>
      {children}
    </SelectLocationViewContext.Provider>
  );
}
