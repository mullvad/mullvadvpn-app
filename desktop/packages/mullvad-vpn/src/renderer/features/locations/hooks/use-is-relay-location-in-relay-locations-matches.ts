import React from 'react';

import type { LiftedConstraint, RelayLocation } from '../../../../shared/daemon-rpc-types';
import { useSettingsRelayLocationsFiltered } from '../../../redux/hooks';
import { LocationType } from '../types';
import { useIsRelayLocationInRelayLocationsForHostnames } from './use-is-relay-location-in-relay-locations-for-hostnames';

export function useIsLocationInRelayLocationsMatches(
  location: LiftedConstraint<RelayLocation>,
  locationType: LocationType,
) {
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();

  const hostnames = React.useMemo(() => {
    const relayLocations =
      locationType === LocationType.exit
        ? relayLocationsFiltered.exit
        : relayLocationsFiltered.entry;

    return relayLocations.matches.map((relayMatch) => relayMatch.relay.hostname);
  }, [locationType, relayLocationsFiltered]);

  const isRelayLocationInRelayLocationsMatches = useIsRelayLocationInRelayLocationsForHostnames(
    location,
    hostnames,
  );

  return isRelayLocationInRelayLocationsMatches;
}
