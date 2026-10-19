import { messages } from '../../../../../shared/gettext';
import {
  useGetIsRelayLocationInRelayLocationsForHostnames,
  useRelayLocationsDiscardsHostnames,
  useRelayLocationsMatchesHostnames,
  useSelectedEntryOrExitLocation,
} from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';

export function useLocationInvalidReason(locationType: LocationType) {
  const location = useSelectedEntryOrExitLocation(locationType);

  const relayLocationsMatchesHostnames = useRelayLocationsMatchesHostnames(locationType);
  const getIsRelayLocationInRelayLocationsMatches =
    useGetIsRelayLocationInRelayLocationsForHostnames(relayLocationsMatchesHostnames);

  const relayLocationsDiscardsHostnames = useRelayLocationsDiscardsHostnames(locationType);
  const getIsRelayLocationInRelayLocationsDiscards =
    useGetIsRelayLocationInRelayLocationsForHostnames(relayLocationsDiscardsHostnames);

  // Ensure we have matches or discards to not give a false positive
  if (relayLocationsMatchesHostnames.length > 0 || relayLocationsDiscardsHostnames.length > 0) {
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
