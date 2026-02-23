import { useMemo } from 'react';

import type { RelayLocation } from '../../../../../shared/daemon-rpc-types';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { LocationType } from '../select-location-types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

// Returns the selected location for the current tunnel protocol and location type
export function useSelectedLocation(): RelayLocation | undefined {
  const { locationType } = useSelectLocationViewContext();
  const relaySettings = useNormalRelaySettings();

  return useMemo(() => {
    if (locationType === LocationType.exit) {
      return relaySettings?.location === 'any' ? undefined : relaySettings?.location;
    } else {
      return relaySettings?.wireguard.entryLocation === 'any'
        ? undefined
        : relaySettings?.wireguard.entryLocation;
    }
  }, [locationType, relaySettings?.location, relaySettings?.wireguard.entryLocation]);
}
