import {
  compareRelayLocationLoose,
  type LiftedConstraint,
  type RelayLocation,
} from '../../../../../shared/daemon-rpc-types';

export function isLocationSelected(
  relayLocation: RelayLocation,
  selected?: LiftedConstraint<RelayLocation>,
) {
  return selected !== 'any' && compareRelayLocationLoose(selected, relayLocation);
}
