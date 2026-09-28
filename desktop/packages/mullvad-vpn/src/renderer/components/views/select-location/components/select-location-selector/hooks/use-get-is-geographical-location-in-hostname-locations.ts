import React from 'react';

import { type RelayLocationGeographical } from '../../../../../../../shared/daemon-rpc-types';
import { useSettingsRelayLocations } from '../../../../../../redux/hooks';
import { getCitiesForHostnames, getCountriesForHostnames, getRelaysForHostnames } from '../utils';

export function useGetIsGeographicalLocationInHostnameLocations(hostnames: string[]) {
  const { relayLocations } = useSettingsRelayLocations();

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

  const getIsGeographicalLocationInHostnameLocations = React.useCallback(
    (location: RelayLocationGeographical) => {
      if ('hostname' in location) {
        return relays.some((relay) => relay.hostname === location.hostname);
      }

      if ('city' in location) {
        return cities.some((city) => city.code === location.city);
      }

      if ('country' in location) {
        return countries.some((country) => country.code === location.country);
      }

      return false;
    },
    [cities, countries, relays],
  );

  return getIsGeographicalLocationInHostnameLocations;
}
