import React from 'react';

import {
  type LocationSelectorProviderProps,
  useLocationSelectorContext,
} from '../../../../LocationSelectorContext';
import type { LocationSelectorItemProps } from './LocationSelectorItem';

type LocationSelectorItemContextProps = Omit<LocationSelectorItemProviderProps, 'children'> & {
  inputRef: React.RefObject<HTMLInputElement | null>;
  selected: boolean;
  setSelected: React.Dispatch<React.SetStateAction<boolean>>;
  inputFocused: boolean;
  setInputFocused: React.Dispatch<React.SetStateAction<boolean>>;
  selectedItem: LocationSelectorProviderProps['selectedItem'];
  onSelectedItemChange: LocationSelectorProviderProps['onSelectedItemChange'];
  onItemInputChange: LocationSelectorProviderProps['onItemInputChange'];
  expanded?: LocationSelectorProviderProps['expanded'];
};

const LocationSelectorItemContext = React.createContext<
  LocationSelectorItemContextProps | undefined
>(undefined);

export const useLocationSelectorItemContext = (): LocationSelectorItemContextProps => {
  const context = React.useContext(LocationSelectorItemContext);
  if (!context) {
    throw new Error(
      'useLocationSelectorItemContext must be used within a LocationSelectorItemProvider',
    );
  }
  return context;
};

type LocationSelectorItemProviderProps = React.PropsWithChildren<{
  id: LocationSelectorItemProps['id'];
  type: LocationSelectorItemProps['type'];
}>;

export function LocationSelectorItemProvider({
  children,
  ...props
}: LocationSelectorItemProviderProps) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const { selectedItem, onSelectedItemChange, onItemInputChange, expanded } =
    useLocationSelectorContext();
  const [selected, setSelected] = React.useState(false);
  const [inputFocused, setInputFocused] = React.useState(false);

  const value = React.useMemo(
    () => ({
      inputRef,
      selected,
      setSelected,
      inputFocused,
      setInputFocused,
      selectedItem,
      onSelectedItemChange,
      onItemInputChange,
      expanded,
      ...props,
    }),
    [
      selected,
      inputFocused,
      selectedItem,
      onSelectedItemChange,
      onItemInputChange,
      expanded,
      props,
    ],
  );

  return (
    <LocationSelectorItemContext.Provider value={value}>
      {children}
    </LocationSelectorItemContext.Provider>
  );
}
