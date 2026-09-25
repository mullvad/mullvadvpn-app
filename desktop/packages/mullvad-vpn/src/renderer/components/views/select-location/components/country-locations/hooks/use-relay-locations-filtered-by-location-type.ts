import { LocationType } from '../../../../../../features/locations/types';
import { useSettingsRelayLocationsFiltered } from '../../../../../../redux/settings/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useRelayLocationsFilteredByLocationType() {
  const { locationType } = useSelectLocationViewContext();
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();

  switch (locationType) {
    case LocationType.entry:
      return relayLocationsFiltered.entry;
    case LocationType.exit:
      return relayLocationsFiltered.exit;
    case LocationType.entryAutomatic:
      return {
        key: 'entryAutomatic',
        matches: [],
        discards: [],
      };
    default:
      return locationType satisfies never;
  }
}
