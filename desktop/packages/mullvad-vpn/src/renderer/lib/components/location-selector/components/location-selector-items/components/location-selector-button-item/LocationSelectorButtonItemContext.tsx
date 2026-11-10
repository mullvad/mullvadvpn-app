import React from 'react';

import {
  type LocationSelectorProviderProps,
  useLocationSelectorContext,
} from '../../../../LocationSelectorContext';
import type { LocationSelectorTextFieldItemProps } from '../location-selector-text-field-item';

type LocationSelectorButtonItemContextProps = Omit<
  LocationSelectorButtonItemProviderProps,
  'children'
> & {
  selectedItem: LocationSelectorProviderProps['selectedItem'];
  onSelectedItemChange: LocationSelectorProviderProps['onSelectedItemChange'];
};

const LocationSelectorButtonItemContext = React.createContext<
  LocationSelectorButtonItemContextProps | undefined
>(undefined);

export const useLocationSelectorButtonItemContext = (): LocationSelectorButtonItemContextProps => {
  const context = React.useContext(LocationSelectorButtonItemContext);
  if (!context) {
    throw new Error(
      'useLocationSelectorButtonItemContext must be used within a LocationSelectorButtonItemProvider',
    );
  }
  return context;
};

type LocationSelectorButtonItemProviderProps = React.PropsWithChildren<{
  id: LocationSelectorTextFieldItemProps['id'];
  type: LocationSelectorTextFieldItemProps['type'];
}>;

export function LocationSelectorButtonItemProvider({
  children,
  id,
  type,
}: LocationSelectorButtonItemProviderProps) {
  const { selectedItem, onSelectedItemChange } = useLocationSelectorContext();

  const value = React.useMemo(
    () => ({
      selectedItem,
      onSelectedItemChange,
      id,
      type,
    }),
    [selectedItem, onSelectedItemChange, id, type],
  );

  return (
    <LocationSelectorButtonItemContext.Provider value={value}>
      {children}
    </LocationSelectorButtonItemContext.Provider>
  );
}
