import React, { useContext, useMemo, useState } from 'react';

import { useNormalRelaySettings } from '../../lib/utilityHooks';
import { RelayListContextProvider } from './RelayListContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { LocationType } from './select-location-types';
import SelectLocation from './SelectLocation';

interface SelectLocationContext {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
}

const selectLocationContext = React.createContext<SelectLocationContext | undefined>(undefined);

export function useSelectLocationContext() {
  return useContext(selectLocationContext)!;
}

export default function SelectLocationContainer() {
  const relaySettings = useNormalRelaySettings();
  const [locationType, setLocationType] = useState(LocationType.exit);

  const [searchTerm, setSearchTerm] = useState('');

  const value = useMemo(() => ({ locationType, setLocationType, searchTerm, setSearchTerm }), [
    locationType,
    relaySettings?.ownership,
    relaySettings?.providers,
    searchTerm,
  ]);

  return (
    <selectLocationContext.Provider value={value}>
      <ScrollPositionContextProvider>
        <RelayListContextProvider>
          <SelectLocation />
        </RelayListContextProvider>
      </ScrollPositionContextProvider>
    </selectLocationContext.Provider>
  );
}
