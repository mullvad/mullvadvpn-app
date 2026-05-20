import React from 'react';

import type { LocationSelectorProps } from './LocationSelector';

type LocationSelectorContextProps = Omit<LocationSelectorProviderProps, 'children'>;

const LocationSelectorContext = React.createContext<LocationSelectorContextProps | undefined>(
  undefined,
);

export const useLocationSelectorContext = (): LocationSelectorContextProps => {
  const context = React.useContext(LocationSelectorContext);
  if (!context) {
    throw new Error('useLocationSelectorContext must be used within a LocationSelectorProvider');
  }
  return context;
};

export type LocationSelectorProviderProps = React.PropsWithChildren<{
  selectedItem: LocationSelectorProps['selectedItem'];
  onSelectedItemChange: LocationSelectorProps['onSelectedItemChange'];
  expanded?: LocationSelectorProps['expanded'];
  variant: LocationSelectorProps['variant'];
}>;

export function LocationSelectorProvider({
  selectedItem,
  onSelectedItemChange,
  expanded,
  variant,
  children,
}: LocationSelectorProviderProps) {
  const value = React.useMemo(
    () => ({
      selectedItem,
      onSelectedItemChange,
      expanded,
      variant,
    }),
    [selectedItem, onSelectedItemChange, expanded, variant],
  );

  return (
    <LocationSelectorContext.Provider value={value}>{children}</LocationSelectorContext.Provider>
  );
}
