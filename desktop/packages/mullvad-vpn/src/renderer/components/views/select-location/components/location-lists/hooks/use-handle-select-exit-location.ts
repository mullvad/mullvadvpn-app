import React from 'react';

import { RoutePath } from '../../../../../../../shared/routes';
import { useAppContext } from '../../../../../../context';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import type { AnyLocation } from '../../../../../../features/locations/types';
import { TransitionType, useHistory } from '../../../../../../lib/history';
import { waitForAnimations } from '../../../../../../lib/utils';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHandleSelectExitLocation() {
  const { exitLocationListsContainerRef, searchTerm, setIsolatedItem, setSearchTerm } =
    useSelectLocationViewContext();
  const { selectExitRelayLocation } = useRelayLocations();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  const handleSelectExitLocation = React.useCallback(
    (location: AnyLocation) => {
      if (!searchTerm) {
        React.startTransition(async () => {
          await selectExitRelayLocation(location.details);
          await waitForAnimations(exitLocationListsContainerRef.current);

          React.startTransition(async () => {
            history.push(RoutePath.main, {
              transition: TransitionType.dismiss,
            });
            await connectTunnel();
          });
        });
      } else {
        // When the user selects an `exit` location from a search we want to
        // wait for the location to be marked as selected in the list before
        // we go to the `main` Route. This is added to mirror the behavior of
        // the `entry` selection handler.
        React.startTransition(async () => {
          await selectExitRelayLocation(location.details);
          await waitForAnimations(exitLocationListsContainerRef.current);
          setSearchTerm('');
          setIsolatedItem(undefined);

          React.startTransition(async () => {
            history.push(RoutePath.main, {
              transition: TransitionType.dismiss,
            });
            await connectTunnel();
          });
        });
      }
    },
    [
      connectTunnel,
      exitLocationListsContainerRef,
      history,
      searchTerm,
      selectExitRelayLocation,
      setIsolatedItem,
      setSearchTerm,
    ],
  );

  return handleSelectExitLocation;
}
