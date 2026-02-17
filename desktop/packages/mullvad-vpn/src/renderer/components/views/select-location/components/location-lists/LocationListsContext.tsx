import React from 'react';

import type { RelayLocation as DaemonRelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { LocationType } from '../../select-location-types';
import { useHandleSelectEntryLocation, useHandleSelectExitLocation } from './hooks';
import type { LocationsListsProps } from './LocationLists';

type LocationListsContextProps = Omit<LocationListsProviderProps, 'children'> & {
  handleSelect: (relayLocation: DaemonRelayLocation) => Promise<void>;
};

const LocationListsContext = React.createContext<LocationListsContextProps | undefined>(undefined);

export const useLocationListsContext = (): LocationListsContextProps => {
  const context = React.useContext(LocationListsContext);
  if (!context) {
    throw new Error('useLocationListsContext must be used within a LocationListsProvider');
  }
  return context;
};

type LocationListsProviderProps = React.PropsWithChildren & {
  type: LocationsListsProps['type'];
};

export function LocationListsProvider({ type, children }: LocationListsProviderProps) {
  const handleSelectExitLocation = useHandleSelectExitLocation();
  const handleSelectEntryLocation = useHandleSelectEntryLocation();

  const handleSelect = React.useMemo(() => {
    if (type === LocationType.entry) {
      return handleSelectEntryLocation;
    }
    return handleSelectExitLocation;
  }, [type, handleSelectEntryLocation, handleSelectExitLocation]);

  const value = React.useMemo(() => ({ type, handleSelect }), [type, handleSelect]);

  return <LocationListsContext.Provider value={value}>{children}</LocationListsContext.Provider>;
}
