import { useSelector } from '../../store';

export function useSettingsRelayLocations() {
  return {
    relayLocations: useSelector((state) => state.settings.relayLocations),
  };
}
