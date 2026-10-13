import { shallowEqual } from 'react-redux';

import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';

export function useRelayLocationsMatchesHostnames(locationType: LocationType) {
  const relayLocationsMatches = useSelector((state) => {
    const context = locationType === LocationType.entry ? 'entry' : 'exit';
    const relayLocationsFiltered = state.settings.relayLocationsFiltered[context];

    return relayLocationsFiltered.matches.map((relayDiscard) => relayDiscard.relay.hostname);
  }, shallowEqual);

  return relayLocationsMatches;
}
