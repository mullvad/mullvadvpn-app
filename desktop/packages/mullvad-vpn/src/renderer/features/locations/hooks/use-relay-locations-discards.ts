import { shallowEqual } from 'react-redux';

import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';

export function useRelayLocationsDiscardsHostnames(locationType: LocationType) {
  const relayLocationsDiscards = useSelector((state) => {
    const context = locationType === LocationType.entry ? 'entry' : 'exit';
    const relayLocationsFiltered = state.settings.relayLocationsFiltered[context];

    return relayLocationsFiltered.discards.map((relayDiscard) => relayDiscard.relay.hostname);
  }, shallowEqual);

  return relayLocationsDiscards;
}
