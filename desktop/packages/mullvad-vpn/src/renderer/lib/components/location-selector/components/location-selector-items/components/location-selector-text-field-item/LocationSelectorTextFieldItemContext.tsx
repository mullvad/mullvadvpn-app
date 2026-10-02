import React from 'react';

import type { LocationSelectorTextFieldItemProps } from './LocationSelectorTextFieldItem';

type LocationSelectorItemContextProps = Omit<
  LocationSelectorTextFieldItemProviderProps,
  'children'
> & {
  inputRef: React.RefObject<HTMLInputElement | null>;
  textFieldRef: React.RefObject<HTMLDivElement | null>;
  triggerRef: React.RefObject<HTMLDivElement | null>;
  focusInsideTextField: boolean;
  setFocusInsideTextField: React.Dispatch<React.SetStateAction<boolean>>;
};

const LocationSelectorTextFieldItemContext = React.createContext<
  LocationSelectorItemContextProps | undefined
>(undefined);

export const useLocationSelectorTextFieldItemContext = (): LocationSelectorItemContextProps => {
  const context = React.useContext(LocationSelectorTextFieldItemContext);
  if (!context) {
    throw new Error(
      'useLocationSelectorTextFieldItemContext must be used within a LocationSelectorTextFieldItemProvider',
    );
  }
  return context;
};

type LocationSelectorTextFieldItemProviderProps = React.PropsWithChildren<{
  id: LocationSelectorTextFieldItemProps['id'];
  type: LocationSelectorTextFieldItemProps['type'];
  inputRef?: LocationSelectorTextFieldItemProps['inputRef'];
  triggerRef?: LocationSelectorTextFieldItemProps['triggerRef'];
}>;

export function LocationSelectorTextFieldItemProvider({
  children,
  inputRef: inputRefProp,
  triggerRef: triggerRefProp,
  ...props
}: LocationSelectorTextFieldItemProviderProps) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const textFieldRef = React.useRef<HTMLDivElement>(null);
  const triggerRef = React.useRef<HTMLDivElement>(null);
  const [focusInsideTextField, setFocusInsideTextField] = React.useState(false);

  const value = React.useMemo(
    () => ({
      inputRef: inputRefProp ?? inputRef,
      triggerRef: triggerRefProp ?? triggerRef,
      textFieldRef,
      focusInsideTextField,
      setFocusInsideTextField,
      ...props,
    }),
    [inputRefProp, triggerRefProp, focusInsideTextField, props],
  );

  return (
    <LocationSelectorTextFieldItemContext.Provider value={value}>
      {children}
    </LocationSelectorTextFieldItemContext.Provider>
  );
}
