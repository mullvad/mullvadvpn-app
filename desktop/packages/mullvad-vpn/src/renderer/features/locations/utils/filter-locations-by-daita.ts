import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { filterLocations } from './filter-locations';
import { isDaitaFilterActive } from './is-daita-filter-active';

export function filterLocationsByDaita(
  locations: IRelayLocationCountryRedux[],
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  return isDaitaFilterActive(daita, directOnly, locationType, multihop)
    ? filterLocations(locations, (relay: IRelayLocationRelayRedux) => relay.daita)
    : locations;
}
