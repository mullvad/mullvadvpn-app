import { useSettingsRelayLocationsFiltered } from '../../../../../../../redux/hooks';
import { useGetIsRelayLocationInRelayLocationsForHostnames } from '../../../../../hooks';
import type { GeographicalLocation } from '../../../../../types';

export function useDisabled(location: GeographicalLocation) {
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const hostnames = relayLocationsFiltered.exit.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );
  const getIsRelayLocationInRelayLocationsForHostnames =
    useGetIsRelayLocationInRelayLocationsForHostnames(hostnames);
  const isRelayLocationInRelayLocationsForHostnames =
    getIsRelayLocationInRelayLocationsForHostnames(location.details);

  const disabled = !isRelayLocationInRelayLocationsForHostnames;

  return disabled;
}
