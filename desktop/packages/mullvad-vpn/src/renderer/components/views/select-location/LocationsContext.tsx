import React from 'react';

import { useObfuscation } from '../../../features/anti-censorship/hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../../features/daita/hooks';
import { useOwnership, useProviders } from '../../../features/locations/hooks';
import { type CountryLocation } from '../../../features/locations/types';
import { filterLocations } from '../../../features/locations/utils';
import { useMultihop } from '../../../features/multihop/hooks';
import { useIpVersion } from '../../../features/tunnel/hooks';
import { useSelector } from '../../../redux/store';
import { useLocations } from './hooks/use-locations';
import { useSearchLocations } from './hooks/use-search-locations';
import { useSelectLocationViewContext } from './SelectLocationViewContext';

type LocationsContextProps = Omit<LocationsProviderProps, 'children'> & {
  filteredLocations: CountryLocation[];
  searchedLocations: CountryLocation[];
};

const LocationsContext = React.createContext<LocationsContextProps | undefined>(undefined);

export const useLocationsContext = (): LocationsContextProps => {
  const context = React.useContext(LocationsContext);
  if (!context) {
    throw new Error('useLocationsContext must be used within a LocationsProvider');
  }
  return context;
};

type LocationsProviderProps = React.PropsWithChildren;

export function LocationsProvider({ children }: LocationsProviderProps) {
  const { locationType, searchTerm } = useSelectLocationViewContext();
  const locations = useSelector((state) => state.settings.relayLocations);
  const { activeOwnership } = useOwnership();
  const { providers } = useProviders();
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { obfuscation } = useObfuscation();
  const { multihop } = useMultihop();
  const { ipVersion } = useIpVersion();

  const filteredRelayLocations = filterLocations({
    locations,
    ownership: activeOwnership,
    providers,
    daita: daitaEnabled,
    directOnly: daitaDirectOnly,
    locationType,
    multihop,
    obfuscation,
    ipVersion,
  });

  const filteredLocations = useLocations(filteredRelayLocations);
  const searchedLocations = useSearchLocations(filteredLocations, searchTerm);

  const value = React.useMemo(
    () => ({
      filteredLocations,
      searchedLocations,
    }),
    [filteredLocations, searchedLocations],
  );

  return <LocationsContext.Provider value={value}>{children}</LocationsContext.Provider>;
}
