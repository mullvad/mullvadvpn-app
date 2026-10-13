import { useSettingsRelayLocationsFiltered } from '../../../../../../../redux/hooks';
import { useIsRelayLocationInRelayLocationsForHostnames } from '../../../../../hooks';
import type { GeographicalLocation } from '../../../../../types';

// TODO: Can we figure out a way to remove this implementation by moving it
// into the `map-redux-*-to-*-location` utility functions?
export function useDisabled(location: GeographicalLocation) {
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const hostnames = relayLocationsFiltered.entry.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );
  const isRelayLocationInRelayLocationsForHostnames =
    useIsRelayLocationInRelayLocationsForHostnames(location.details, hostnames);

  const disabled = !isRelayLocationInRelayLocationsForHostnames;

  return disabled;
}
