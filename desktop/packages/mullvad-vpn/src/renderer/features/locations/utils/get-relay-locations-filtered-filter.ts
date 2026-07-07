import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import {
  type IRelayLocationRelayRedux,
  type RelayLocationsFilterContext,
  type RelayLocationsFiltered,
} from '../../../redux/settings/reducers';
import { getRelayLocationsFilteredDiscardsFilter } from './get-relay-locations-filtered-discards-filter';
import { getRelayLocationsFilteredMatchesFilter } from './get-relay-locations-filtered-matches-filter';

export function getRelayLocationsFilteredFilter(
  relayLocationsFiltered: RelayLocationsFiltered,
  context: RelayLocationsFilterContext,
  multihop: MultihopMode,
): (relay: IRelayLocationRelayRedux) => boolean {
  return (relay) => {
    const { discards, matches } = relayLocationsFiltered[context];
    const filters = [
      getRelayLocationsFilteredMatchesFilter(matches),
      getRelayLocationsFilteredDiscardsFilter(discards, multihop),
    ];

    return filters.some((filter) => filter(relay));
  };
}
