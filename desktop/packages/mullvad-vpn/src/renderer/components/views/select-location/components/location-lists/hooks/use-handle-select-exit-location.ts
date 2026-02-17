import React from 'react';

import { type RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../../../../context';
import { useSelectLocation } from '../../../../../../features/location/hooks';
import { useHistory } from '../../../../../../lib/history';

export function useHandleSelectExitLocation() {
  const { selectExitLocation } = useSelectLocation();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    async (relayLocation: RelayLocation) => {
      history.pop();
      await selectExitLocation(relayLocation);
      await connectTunnel();
    },
    [connectTunnel, history, selectExitLocation],
  );

  return handleSelectExitLocation;
}
