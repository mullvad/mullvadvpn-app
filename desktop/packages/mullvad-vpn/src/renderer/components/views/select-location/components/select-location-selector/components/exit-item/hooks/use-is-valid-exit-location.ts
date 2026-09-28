import { useSelectedLocations } from '../../../../../../../../features/locations/hooks';
import { useSettingsRelayLocationsFiltered } from '../../../../../../../../redux/hooks';
import { useIsLocationInHostnameLocations } from '../../../hooks';

export function useIsValidExitLocation() {
  const { exit } = useSelectedLocations();

  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const hostnames = relayLocationsFiltered.exit.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );

  const isLocationInHostnameLocations = useIsLocationInHostnameLocations(exit, hostnames);

  return isLocationInHostnameLocations;
}
