import React from 'react';

import type { LiftedConstraint, RelayLocation } from '../../../../shared/daemon-rpc-types';
import { isCustomList } from '../utils';
import { useGetIsCustomListInRelayLocationsForHostnames } from './use-get-is-custom-list-in-relay-locations-for-hostnames';
import { useGetIsGeographicalLocationInRelayLocationsForHostnames } from './use-get-is-geographical-location-in-relay-locations-for-hostnames';

export function useGetIsRelayLocationInRelayLocationsForHostnames(hostnames: string[]) {
  const getIsGeographicalLocationInRelayLocationsForHostnames =
    useGetIsGeographicalLocationInRelayLocationsForHostnames(hostnames);
  const getIsCustomListInRelayLocationsForHostnames =
    useGetIsCustomListInRelayLocationsForHostnames(hostnames);

  const getIsRelayLocationInRelayLocationsForHostnames = React.useCallback(
    (location: LiftedConstraint<RelayLocation>) => {
      // If 'any' is used we can consider it to be in the list of hostnames immediately
      // as long as the hostnames array contains at least one entry.
      if (location === 'any') {
        return hostnames.length > 0;
      }

      if (isCustomList(location)) {
        return getIsCustomListInRelayLocationsForHostnames(location);
      }

      return getIsGeographicalLocationInRelayLocationsForHostnames(location);
    },
    [
      getIsCustomListInRelayLocationsForHostnames,
      getIsGeographicalLocationInRelayLocationsForHostnames,
      hostnames.length,
    ],
  );

  return getIsRelayLocationInRelayLocationsForHostnames;
}
