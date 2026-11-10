import React from 'react';

import { RelayLocationCustomList } from '../../../../shared/daemon-rpc-types';
import { useSettingsCustomLists } from '../../../redux/hooks';
import { findCustomList } from '../utils';
import { useGetIsGeographicalLocationInRelayLocationsForHostnames } from './use-get-is-geographical-location-in-relay-locations-for-hostnames';

export function useGetIsCustomListInRelayLocationsForHostnames(hostnames: string[]) {
  const { customLists } = useSettingsCustomLists();
  const getIsGeographicalLocationInHostnameLocations =
    useGetIsGeographicalLocationInRelayLocationsForHostnames(hostnames);

  const getIsCustomListInRelayLocationsForHostnames = React.useCallback(
    (location: RelayLocationCustomList) => {
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

  return getIsCustomListInRelayLocationsForHostnames;
}
