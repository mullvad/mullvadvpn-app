import React from 'react';

import { useSettingsRelayLocationsFiltered } from '../../../redux/hooks';
import { LocationType } from '../types';

export function useRelayLocationsDiscards(locationType: LocationType) {
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();

  const relayLocationsDiscards = React.useMemo(() => {
    const relayLocations =
      locationType === LocationType.exit
        ? relayLocationsFiltered.exit
        : relayLocationsFiltered.entry;

    return relayLocations.discards;
  }, [locationType, relayLocationsFiltered]);

  return relayLocationsDiscards;
}
