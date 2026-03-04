import { Ownership } from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getOwnershipFilter(
  ownership?: Ownership,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  const filterActive = ownership !== undefined && ownership !== Ownership.any;

  const expectOwned = ownership === Ownership.mullvadOwned;
  return filterActive ? (relay) => relay.owned === expectOwned : undefined;
}
