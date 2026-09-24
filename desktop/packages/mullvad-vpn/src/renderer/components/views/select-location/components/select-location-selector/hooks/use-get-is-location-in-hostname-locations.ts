import React from 'react';

import {
  RelayLocation,
  RelayLocationCustomList,
  RelayLocationGeographical,
} from '../../../../../../../shared/daemon-rpc-types';
import { useCustomLists } from '../../../../../../features/custom-lists/hooks';
import {
  findCityForRelay,
  findCountryForRelay,
  findCustomList,
  findRelay,
} from '../../../../../../features/locations/utils';
import { useSettingsRelayLocations } from '../../../../../../redux/hooks';
import { IRelayLocationCountryRedux } from '../../../../../../redux/settings/reducers';

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

export function useGetIsCustomListInHostnameLocations(hostnames: string[]) {
  const { customLists } = useCustomLists();
  const getIsGeographicalLocationInHostnameLocations =
    useGetIsGeographicalLocationInHostnameLocations(hostnames);

  const getIsCustomListInHostnameLocations = React.useCallback(
    (location: Partial<RelayLocationCustomList>) => {
      if (location.customList) {
        const customList = findCustomList(location.customList, customLists);
        if (customList) {
          return customList.locations.some((location) =>
            getIsGeographicalLocationInHostnameLocations(location),
          );
        }
      }

      return false;
    },
    [customLists, getIsGeographicalLocationInHostnameLocations],
  );

  return getIsCustomListInHostnameLocations;
}

export function useGetIsLocationInHostnameLocations(hostnames: string[]) {
  const getIsGeographicalLocationInHostnameLocations =
    useGetIsGeographicalLocationInHostnameLocations(hostnames);
  const getIsCustomListInHostnameLocations = useGetIsCustomListInHostnameLocations(hostnames);

  const getIsLocationInHostnameLocations = React.useCallback(
    (location: RelayLocation) => {
      if ('customList' in location) {
        return getIsCustomListInHostnameLocations(location);
      }

      return getIsGeographicalLocationInHostnameLocations(location);
    },
    [getIsCustomListInHostnameLocations, getIsGeographicalLocationInHostnameLocations],
  );

  return getIsLocationInHostnameLocations;
}

function getRelaysForHostnames(hostnames: string[], locations: IRelayLocationCountryRedux[]) {
  const relays = hostnames
    .map((hostname) => findRelay(hostname, locations))
    .filter((relay) => relay !== undefined);

  return relays; // TODO: remove duplicates
}

function getCitiesForHostnames(hostnames: string[], locations: IRelayLocationCountryRedux[]) {
  const cities = hostnames
    .map((hostname) => findCityForRelay(hostname, locations))
    .filter((city) => city !== undefined);

  return cities; // TODO: remove duplicates
}

function getCountriesForHostnames(hostnames: string[], locations: IRelayLocationCountryRedux[]) {
  const countries = hostnames
    .map((hostname) => findCountryForRelay(hostname, locations))
    .filter((relay) => relay !== undefined);

  return countries; // TODO: remove duplicates
}
