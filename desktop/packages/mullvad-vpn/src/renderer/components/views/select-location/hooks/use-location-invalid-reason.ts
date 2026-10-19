import React from 'react';

import { messages } from '../../../../../shared/gettext';
import {
  useGetIsRelayLocationInRelayLocationsForHostnames,
  useRelayLocationsDiscards,
  useRelayLocationsMatches,
  useSelectedEntryOrExitLocation,
} from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';

export function useLocationInvalidReason(locationType: LocationType) {
  const location = useSelectedEntryOrExitLocation(locationType);

  const relayLocationsMatches = useRelayLocationsMatches(locationType);
  const relayLocationsMatchesHostnames = React.useMemo(
    () => relayLocationsMatches.map((relay) => relay.relay.hostname),
    [relayLocationsMatches],
  );
  const getIsRelayLocationInRelayLocationsMatches =
    useGetIsRelayLocationInRelayLocationsForHostnames(relayLocationsMatchesHostnames);

  const relayLocationsDiscards = useRelayLocationsDiscards(locationType);
  const relayLocationsDiscardsHostnames = React.useMemo(
    () => relayLocationsDiscards.map((relay) => relay.relay.hostname),
    [relayLocationsDiscards],
  );
  const getIsRelayLocationInRelayLocationsDiscards =
    useGetIsRelayLocationInRelayLocationsForHostnames(relayLocationsDiscardsHostnames);

  // Ensure we have matches or discards to not give a false positive
  if (relayLocationsMatches.length > 0 || relayLocationsDiscards.length > 0) {
    const isLocationInRelayLocationsMatches = getIsRelayLocationInRelayLocationsMatches(location);
    const isLocationInRelayLocationsDiscards = getIsRelayLocationInRelayLocationsDiscards(location);

    // Selected location neither exists in the list of available relay locations, as either
    // a discarded or a matched relay
    if (!isLocationInRelayLocationsDiscards && !isLocationInRelayLocationsMatches) {
      // TRANSLATORS: Error label informing the user that their currently selected server
      // TRANSLATORS: does not match their current filter settings
      return messages.pgettext('select-location-view', 'Selection is unavailable');
    }

    // Selected location doesn't exist in the list of available relay locations matches
    if (!isLocationInRelayLocationsMatches) {
      // TRANSLATORS: Error label informing the user that their currently selected server
      // TRANSLATORS: does not match their current filter settings
      return messages.pgettext('select-location-view', 'Selection does not match filter settings');
    }
  }

  return undefined;
}
