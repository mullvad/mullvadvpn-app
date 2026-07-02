import React from 'react';

import {
  type LocationSelectorProviderProps,
  useLocationSelectorContext,
} from '../../../../LocationSelectorContext';
import type { LocationSelectorItemProps } from './LocationSelectorItem';

type LocationSelectorItemContextProps = Omit<LocationSelectorItemProviderProps, 'children'> & {
  inputRef: React.RefObject<HTMLInputElement | null>;
  textFieldRef: React.RefObject<HTMLDivElement | null>;
  triggerRef: React.RefObject<HTMLDivElement | null>;
  focusInsideTextField: boolean;
  setFocusInsideTextField: React.Dispatch<React.SetStateAction<boolean>>;
  selectedItem: LocationSelectorProviderProps['selectedItem'];
  onSelectedItemChange: LocationSelectorProviderProps['onSelectedItemChange'];
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
  inputRef?: LocationSelectorItemProps['inputRef'];
  triggerRef?: LocationSelectorItemProps['triggerRef'];
}>;

export function LocationSelectorItemProvider({
  children,
  inputRef: inputRefProp,
  triggerRef: triggerRefProp,
  ...props
}: LocationSelectorItemProviderProps) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const textFieldRef = React.useRef<HTMLDivElement>(null);
  const triggerRef = React.useRef<HTMLDivElement>(null);
  const { selectedItem, onSelectedItemChange, expanded } = useLocationSelectorContext();
  const [focusInsideTextField, setFocusInsideTextField] = React.useState(false);

  const value = React.useMemo(
    () => ({
      inputRef: inputRefProp ? inputRefProp : inputRef,
      triggerRef: triggerRefProp ? triggerRefProp : triggerRef,
      textFieldRef,
      selectedItem,
      onSelectedItemChange,
      expanded,
      focusInsideTextField,
      setFocusInsideTextField,
      ...props,
    }),
    [
      inputRefProp,
      triggerRefProp,
      selectedItem,
      onSelectedItemChange,
      expanded,
      focusInsideTextField,
      props,
    ],
  );

  return (
    <LocationSelectorItemContext.Provider value={value}>
      {children}
    </LocationSelectorItemContext.Provider>
  );
}
