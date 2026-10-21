import { shallowEqual } from 'react-redux';

import { useSelector } from '../../store';

export function useSettingsRelayLocationsFiltered() {
  return {
    relayLocationsFiltered: useSelector(
      (state) => state.settings.relayLocationsFiltered,
      shallowEqual,
    ),
  };
}
