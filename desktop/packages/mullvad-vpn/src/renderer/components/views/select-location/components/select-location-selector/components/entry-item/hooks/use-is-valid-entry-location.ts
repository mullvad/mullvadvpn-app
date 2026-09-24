import { useSelectedLocations } from '../../../../../../../../features/locations/hooks';
import { useSettingsRelayLocationsFiltered } from '../../../../../../../../redux/hooks';
import { useGetIsLocationInHostnameLocations } from '../../../hooks';

export function useIsValidEntryLocation() {
  const { entry } = useSelectedLocations();

  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const relayLocationsFilteredHostnames = relayLocationsFiltered.entry.matches.map(
    (relayMatch) => relayMatch.relay.hostname,
  );

  const getIsLocationInHostnameLocations = useGetIsLocationInHostnameLocations(
    relayLocationsFilteredHostnames,
  );

  if (entry === 'any') {
    return true;
  }

  return getIsLocationInHostnameLocations(entry);
}
