import { useSettingsRelayLocationsFiltered } from '../../../../../../redux/settings/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useRelayLocationsFilteredByLocationType() {
  const { locationType } = useSelectLocationViewContext();
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();

  switch (locationType) {
    case 'entry':
      return relayLocationsFiltered.entry;
    case 'exit':
      return relayLocationsFiltered.exit;
    case 'automaticEntry':
      return {
        key: 'automaticEntry',
        matches: [],
        discards: [],
      };
    default:
      return locationType satisfies never;
  }
}
