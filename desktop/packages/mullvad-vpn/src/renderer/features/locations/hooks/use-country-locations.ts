import React from 'react';

import { useSelector } from '../../../redux/store';
import { useObfuscation } from '../../anti-censorship/hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../daita/hooks';
import { useMultihop } from '../../multihop/hooks';
import { useIpVersion } from '../../tunnel/hooks';
import { type LocationType } from '../types';
import { filterLocations } from '../utils';
import { useOwnership, useProviders } from '.';
import { useMapReduxCountriesToCountryLocations } from './use-map-redux-countries-to-country-locations';
import { useSearchCountryLocations } from './use-search-country-locations';

export function useCountryLocations(locationType: LocationType, searchTerm: string) {
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

  const filteredLocations = useMapReduxCountriesToCountryLocations(
    filteredRelayLocations,
    locationType,
  );
  const searchedLocations = useSearchCountryLocations(filteredLocations, searchTerm);

  return React.useMemo(
    () => ({
      filteredLocations,
      searchedLocations,
    }),
    [filteredLocations, searchedLocations],
  );
}
