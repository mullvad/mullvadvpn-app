import { compareRelayLocation, type RelayLocationRelay } from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';
import { useDisabledLocation } from '../hooks';
import { DisabledReason } from '../types';

export function isRelayDisabled(
  relay: IRelayLocationRelayRedux,
  location: RelayLocationRelay,
  disabledLocation?: ReturnType<typeof useDisabledLocation>,
): DisabledReason | undefined {
  if (!relay.active) {
    return DisabledReason.inactive;
  } else if (
    disabledLocation?.location &&
    compareRelayLocation(location, disabledLocation.location)
  ) {
    return disabledLocation.reason;
  } else {
    return undefined;
  }
}
