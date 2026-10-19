import { shallowEqual } from 'react-redux';

import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';

export function useRelayLocationsMatchesHostnames(locationType: LocationType) {
  const relayLocationsMatches = useSelector(
    (state) =>
      state.settings.relayLocationsFiltered[
        locationType === LocationType.entry ? 'entry' : 'exit'
      ].matches.map((relayMatch) => relayMatch.relay.hostname),
    shallowEqual,
  );

  return relayLocationsMatches;
}
