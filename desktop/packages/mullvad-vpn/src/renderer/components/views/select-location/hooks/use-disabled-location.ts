import { useMemo } from 'react';

import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { DisabledReason, LocationType } from '../select-location-types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

// Returns the location (if any) that should be disabled. This is currently used for disabling the
// entry location when selecting exit location etc.
export function useDisabledLocation() {
  const { locationType } = useSelectLocationViewContext();
  const relaySettings = useNormalRelaySettings();

  return useMemo(() => {
    if (relaySettings?.wireguard.useMultihop) {
      if (locationType === LocationType.exit && relaySettings?.wireguard.entryLocation !== 'any') {
        return {
          location: relaySettings?.wireguard.entryLocation,
          reason: DisabledReason.entry,
        };
      } else if (locationType === LocationType.entry && relaySettings?.location !== 'any') {
        return { location: relaySettings?.location, reason: DisabledReason.exit };
      }
    }

    return undefined;
  }, [
    locationType,
    relaySettings?.wireguard.useMultihop,
    relaySettings?.wireguard.entryLocation,
    relaySettings?.location,
  ]);
}
