import { useSelectedLocations } from '../../../../../../../../features/locations/hooks';
import { useSettingsRelayLocationsFiltered } from '../../../../../../../../redux/hooks';
import { useIsLocationInHostnameLocations } from '../../../hooks';

export function useIsValidEntryLocation() {
  const { entry } = useSelectedLocations();

  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const hostnames = relayLocationsFiltered.entry.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );

  const isLocationInHostnameLocations = useIsLocationInHostnameLocations(entry, hostnames);

  return isLocationInHostnameLocations;
}
