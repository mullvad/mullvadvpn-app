import React from 'react';

import { ObfuscationType } from '../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../../features/anti-censorship/hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../../features/daita/hooks';
import {
  filterLocations as filterLocationsByOwnershipAndProviders,
  filterLocationsByDaita,
  filterLocationsByLwo,
  filterLocationsByQuic,
} from '../../../lib/filter-locations';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useSelector } from '../../../redux/store';
import { useLocations } from './hooks/use-locations';
import { useSearchLocations } from './hooks/use-search-locations';
import { type CountryLocation } from './select-location-types';
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
  const relayLocations = useSelector((state) => state.settings.relayLocations);
  const relaySettings = useNormalRelaySettings();
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { obfuscation } = useObfuscation();
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const ipVersion = relaySettings?.wireguard.ipVersion ?? 'any';
  const quic = obfuscation === ObfuscationType.quic;
  const lwo = obfuscation === ObfuscationType.lwo;

  const daitaRelayLocations = filterLocationsByDaita(
    relayLocations,
    daitaEnabled,
    daitaDirectOnly,
    locationType,
    multihop,
  );

  const quicRelayLocations = filterLocationsByQuic(
    daitaRelayLocations,
    quic,
    locationType,
    multihop,
    ipVersion,
  );

  const lwoRelayLocations = filterLocationsByLwo(quicRelayLocations, lwo, locationType, multihop);

  // Filters locations on ownership and providers
  const filteredRelayLocations = filterLocationsByOwnershipAndProviders(
    lwoRelayLocations,
    relaySettings?.ownership,
    relaySettings?.providers,
  );

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
