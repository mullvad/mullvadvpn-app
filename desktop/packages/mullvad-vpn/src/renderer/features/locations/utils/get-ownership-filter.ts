import { Ownership } from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getOwnershipFilter(
  ownership?: Ownership,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  if (ownership === undefined || ownership === Ownership.any) {
    return undefined;
  }

  const expectOwned = ownership === Ownership.mullvadOwned;
  return (relay) => relay.owned === expectOwned;
}
