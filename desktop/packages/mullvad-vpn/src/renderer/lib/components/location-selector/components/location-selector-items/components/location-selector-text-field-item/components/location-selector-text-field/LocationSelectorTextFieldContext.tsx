import React from 'react';

import { useLocationSelectorContext } from '../../../../../../LocationSelectorContext';
import type { LocationSelectorTextFieldItemProps } from '../../LocationSelectorTextFieldItem';

type LocationSelectorTextFieldContextProps = Omit<
  LocationSelectorTextFieldProviderProps,
  'children'
> & {
  inputRef: React.RefObject<HTMLInputElement | null>;
  textFieldRef: React.RefObject<HTMLDivElement | null>;
  triggerRef: React.RefObject<HTMLDivElement | null>;
  focusInsideTextField: boolean;
  setFocusInsideTextField: React.Dispatch<React.SetStateAction<boolean>>;
};

const LocationSelectorTextFieldContext = React.createContext<
  LocationSelectorTextFieldContextProps | undefined
>(undefined);

export const useLocationSelectorTextFieldContext = (): LocationSelectorTextFieldContextProps => {
  const context = React.useContext(LocationSelectorTextFieldContext);
  if (!context) {
    throw new Error(
      'useLocationSelectorTextFieldContext must be used within a LocationSelectorTextFieldProvider',
    );
  }
  return context;
};

type LocationSelectorTextFieldProviderProps = React.PropsWithChildren<{
  inputRef?: LocationSelectorTextFieldItemProps['inputRef'];
  triggerRef?: LocationSelectorTextFieldItemProps['triggerRef'];
}>;

export function LocationSelectorTextFieldProvider({
  children,
  inputRef: inputRefProp,
  triggerRef: triggerRefProp,
  ...props
}: LocationSelectorTextFieldProviderProps) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const textFieldRef = React.useRef<HTMLDivElement>(null);
  const triggerRef = React.useRef<HTMLDivElement>(null);
  const { selectedItem, onSelectedItemChange, expanded } = useLocationSelectorContext();
  const [focusInsideTextField, setFocusInsideTextField] = React.useState(false);

  const value = React.useMemo(
    () => ({
      inputRef: inputRefProp ?? inputRef,
      triggerRef: triggerRefProp ?? triggerRef,
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
    <LocationSelectorTextFieldContext.Provider value={value}>
      {children}
    </LocationSelectorTextFieldContext.Provider>
  );
}
