import React from 'react';

import { RoutePath } from '../../../../../../../shared/routes';
import { useAppContext } from '../../../../../../context';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import type { AnyLocation } from '../../../../../../features/locations/types';
import { useHistory } from '../../../../../../lib/history';

export function useHandleSelectExitLocation() {
  const { selectExitRelayLocation } = useRelayLocations();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    async (location: AnyLocation) => {
      history.push(RoutePath.main);
      await selectExitRelayLocation(location.details);
      await connectTunnel();
    },
    [connectTunnel, history, selectExitRelayLocation],
  );

  return handleSelectExitLocation;
}
