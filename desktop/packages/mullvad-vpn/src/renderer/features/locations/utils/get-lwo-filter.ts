import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { isLwoFilterActive } from './is-lwo-filter-active';

export function getLwoFilter(
  lwo: boolean,
  locationType: LocationType,
  multihop: boolean,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  const filterActive = isLwoFilterActive(lwo, locationType, multihop);
  return filterActive ? (relay: IRelayLocationRelayRedux) => relay.lwo : undefined;
}
