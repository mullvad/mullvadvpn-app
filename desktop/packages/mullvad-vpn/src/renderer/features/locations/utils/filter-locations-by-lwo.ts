import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { filterLocations } from './filter-locations';
import { isLwoFilterActive } from './is-lwo-filter-active';

export function filterLocationsByLwo(
  locations: IRelayLocationCountryRedux[],
  lwo: boolean,
  locationType: LocationType,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  const lwoOnRelay = (relay: IRelayLocationRelayRedux) => relay.lwo;
  return isLwoFilterActive(lwo, locationType, multihop)
    ? filterLocations(locations, lwoOnRelay)
    : locations;
}
