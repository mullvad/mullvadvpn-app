import React from 'react';

import { type RelayLocationGeographical } from '../../../../shared/daemon-rpc-types';
import { isCity, isCountry, isRelay } from '../utils';
import { getCitiesForHostnames, getCountriesForHostnames, getRelaysForHostnames } from '../utils';
import { useRelayLocations } from './use-relay-locations';

export function useGetIsGeographicalLocationInRelayLocationsForHostnames(hostnames: string[]) {
  const { relayLocations } = useRelayLocations();

  const relays = React.useMemo(
    () => getRelaysForHostnames(hostnames, relayLocations),
    [relayLocations, hostnames],
  );
  const cities = React.useMemo(
    () => getCitiesForHostnames(hostnames, relayLocations),
    [relayLocations, hostnames],
  );
  const countries = React.useMemo(
    () => getCountriesForHostnames(hostnames, relayLocations),
    [relayLocations, hostnames],
  );

  const getIsGeographicalLocationInRelayLocationsForHostnames = React.useCallback(
    (location: RelayLocationGeographical) => {
      if (isRelay(location)) {
        return relays.some((relay) => relay.hostname === location.hostname);
      }

      if (isCity(location)) {
        return cities.some((city) => city.code === location.city);
      }

      if (isCountry(location)) {
        return countries.some((country) => country.code === location.country);
      }

      return false;
    },
    [cities, countries, relays],
  );

  return getIsGeographicalLocationInRelayLocationsForHostnames;
}
