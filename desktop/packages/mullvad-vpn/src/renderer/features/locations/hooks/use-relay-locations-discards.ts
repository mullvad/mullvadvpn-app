import { useSelector } from '../../../redux/store';
import { LocationType } from '../types';

export function useRelayLocationsDiscardsHostnames(locationType: LocationType) {
  const relayLocationsDiscards = useSelector((state) =>
    state.settings.relayLocationsFiltered[
      locationType === LocationType.entry ? 'entry' : 'exit'
    ].discards.map((relayDiscard) => relayDiscard.relay.hostname),
  );

  return relayLocationsDiscards;
}
