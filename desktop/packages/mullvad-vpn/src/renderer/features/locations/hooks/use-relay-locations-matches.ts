import React from 'react';

import { useSettingsRelayLocationsFiltered } from '../../../redux/hooks';
import { LocationType } from '../types';

export function useRelayLocationsMatches(locationType: LocationType) {
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();

  const relayLocationsMatches = React.useMemo(() => {
    const relayLocations =
      locationType === LocationType.exit
        ? relayLocationsFiltered.exit
        : relayLocationsFiltered.entry;

    return relayLocations.matches;
  }, [locationType, relayLocationsFiltered]);

  return relayLocationsMatches;
}
