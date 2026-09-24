import { useSelectedLocations } from '../../../../../../../../features/locations/hooks';
import { useSettingsRelayLocationsFiltered } from '../../../../../../../../redux/hooks';
import { useGetIsLocationInHostnameLocations } from '../../../hooks';

export function useIsValidExitLocation() {
  const { exit } = useSelectedLocations();

  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const relayLocationsFilteredHostnames = relayLocationsFiltered.exit.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );

  const getIsLocationInHostnameLocations = useGetIsLocationInHostnameLocations(
    relayLocationsFilteredHostnames,
  );

  if (exit === 'any') {
    return true;
  }

  return getIsLocationInHostnameLocations(exit);
}
