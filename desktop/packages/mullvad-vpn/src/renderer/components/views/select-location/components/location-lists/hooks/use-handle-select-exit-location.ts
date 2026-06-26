import React from 'react';

import { RoutePath } from '../../../../../../../shared/routes';
import { useAppContext } from '../../../../../../context';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import type { AnyLocation } from '../../../../../../features/locations/types';
import { TransitionType, useHistory } from '../../../../../../lib/history';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectExitLocation() {
  const { searchTerm } = useSelectLocationViewContext();
  const { selectExitRelayLocation } = useRelayLocations();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    async (location: AnyLocation) => {
      if (!searchTerm) {
        history.push(RoutePath.main, {
          transition: TransitionType.dismiss,
        });
        await connectTunnel();
      }
      await selectExitRelayLocation(location.details);
    },
    [connectTunnel, history, searchTerm, selectExitRelayLocation],
  );

  return handleSelectExitLocation;
}
