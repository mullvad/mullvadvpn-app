import React, { startTransition } from 'react';

import { RoutePath } from '../../../../../../../shared/routes';
import { useAppContext } from '../../../../../../context';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import type { AnyLocation } from '../../../../../../features/locations/types';
import { TransitionType, useHistory } from '../../../../../../lib/history';
import { waitForAnimations } from '../../../../../../lib/utils';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectExitLocation() {
  const { exitLocationListsContainerRef, searchTerm } = useSelectLocationViewContext();
  const { selectExitRelayLocation } = useRelayLocations();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    async (location: AnyLocation) => {
      if (!searchTerm) {
        history.push(RoutePath.main, {
          transition: TransitionType.dismiss,
        });
        await selectExitRelayLocation(location.details);
        await connectTunnel();
      } else {
        // When the user selects an `exit` location from a search we want to
        // wait for the location to be marked as selected in the list before
        // we go to the `main` Route. This is added to mirror the behavior of
        // the `entry` selection handler.
        startTransition(async () => {
          await selectExitRelayLocation(location.details);
          await waitForAnimations(exitLocationListsContainerRef.current);
          history.push(RoutePath.main, {
            transition: TransitionType.dismiss,
          });
          await connectTunnel();
        });
      }
    },
    [connectTunnel, exitLocationListsContainerRef, history, searchTerm, selectExitRelayLocation],
  );

  return handleSelectExitLocation;
}
