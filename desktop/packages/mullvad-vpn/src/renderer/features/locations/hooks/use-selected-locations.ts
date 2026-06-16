import React from 'react';

import type { RelayLocation } from '../../../../shared/daemon-rpc-types';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

// Returns selected entry and exit locations.
export function useSelectedLocations(): Record<'entry' | 'exit', RelayLocation | undefined> {
  const relaySettings = useNormalRelaySettings();

  return React.useMemo(() => {
    const exit = relaySettings?.location === 'any' ? undefined : relaySettings?.location;
    const entry =
      relaySettings?.wireguard.entryLocation === 'any'
        ? undefined
        : relaySettings?.wireguard.entryLocation;
    return { entry, exit };
  }, [relaySettings?.location, relaySettings?.wireguard.entryLocation]);
}
