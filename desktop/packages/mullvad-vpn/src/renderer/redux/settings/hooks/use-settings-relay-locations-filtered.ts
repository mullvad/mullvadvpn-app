import { useSelector } from '../../store';

export function useSettingsRelayLocationsFiltered() {
  return {
    relayLocationsFiltered: useSelector((state) => state.settings.relayLocationsFiltered),
  };
}
