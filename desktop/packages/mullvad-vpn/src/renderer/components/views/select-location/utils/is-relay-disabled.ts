import {
  compareRelayLocation,
  type RelayLocation,
  type RelayLocationRelay,
} from '../../../../../shared/daemon-rpc-types';
import type { IRelayLocationRelayRedux } from '../../../../redux/settings/reducers';
import { DisabledReason } from '../select-location-types';

export function isRelayDisabled(
  relay: IRelayLocationRelayRedux,
  location: RelayLocationRelay,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
): DisabledReason | undefined {
  if (!relay.active) {
    return DisabledReason.inactive;
  } else if (disabledLocation && compareRelayLocation(location, disabledLocation.location)) {
    return disabledLocation.reason;
  } else {
    return undefined;
  }
}
