import { Ownership } from '../../../../shared/daemon-rpc-types';

export function isOwnershipFilterActive(ownership: Ownership) {
  return ownership !== Ownership.any;
}
