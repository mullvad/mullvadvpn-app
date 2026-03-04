import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { isDaitaFilterActive } from './is-daita-filter-active';

export function getDaitaFilter(
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  multihop: boolean,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  const filterActive = isDaitaFilterActive(daita, directOnly, locationType, multihop);
  return filterActive ? (relay: IRelayLocationRelayRedux) => relay.daita : undefined;
}
