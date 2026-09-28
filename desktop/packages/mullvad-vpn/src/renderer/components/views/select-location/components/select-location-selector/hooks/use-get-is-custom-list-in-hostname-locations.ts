import React from 'react';

import { RelayLocationCustomList } from '../../../../../../../shared/daemon-rpc-types';
import { useCustomLists } from '../../../../../../features/custom-lists/hooks';
import { findCustomList } from '../../../../../../features/locations/utils';
import { useGetIsGeographicalLocationInHostnameLocations } from './use-get-is-geographical-location-in-hostname-locations';

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
