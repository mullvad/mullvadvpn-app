import React from 'react';

import type { LiftedConstraint, RelayLocation } from '../../../../shared/daemon-rpc-types';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

// Returns selected entry and exit locations.
export function useSelectedLocations(): {
  exit: LiftedConstraint<RelayLocation>;
  entry: LiftedConstraint<RelayLocation>;
} {
  const relaySettings = useNormalRelaySettings();

  return React.useMemo(() => {
    if (!relaySettings) {
      return { entry: 'any', exit: 'any' };
    }

    const exit = relaySettings.location;
    const entry = relaySettings.wireguard.entryLocation;
    return { entry, exit };
  }, [relaySettings]);
}
