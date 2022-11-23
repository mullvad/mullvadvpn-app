import React, { useContext, useMemo, useState } from 'react';

import { RelayListContextProvider } from './RelayListContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { LocationType } from './select-location-types';
import SelectLocation from './SelectLocation';

// Context containing data required by different components in the sub tree
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
  const [locationType, setLocationType] = useState(LocationType.exit);
  const [searchTerm, setSearchTerm] = useState('');

  const value = useMemo(() => ({ locationType, setLocationType, searchTerm, setSearchTerm }), [
    locationType,
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
