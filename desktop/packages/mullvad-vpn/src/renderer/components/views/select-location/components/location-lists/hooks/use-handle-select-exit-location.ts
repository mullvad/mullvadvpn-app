import React from 'react';

import { type RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../../../../context';
import { useRelayLocations } from '../../../../../../features/location/hooks';
import { useHistory } from '../../../../../../lib/history';

export function useHandleSelectExitLocation() {
  const { selectExitRelayLocation } = useRelayLocations();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    async (relayLocation: RelayLocation) => {
      history.pop();
      await selectExitRelayLocation(relayLocation);
      await connectTunnel();
    },
    [connectTunnel, history, selectExitRelayLocation],
  );

  return handleSelectExitLocation;
}
