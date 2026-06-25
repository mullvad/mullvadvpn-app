import { compareRelayLocation, type RelayLocationCity } from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationCityRedux } from '../../../redux/settings/reducers';
import { useDisabledLocation } from '../hooks';
import { DisabledReason } from '../types';
import { isRelayDisabled } from './is-relay-disabled';

export function isCityDisabled(
  city: IRelayLocationCityRedux,
  location: RelayLocationCity,
  disabledLocation?: ReturnType<typeof useDisabledLocation>,
): DisabledReason | undefined {
  const relaysDisabled = city.relays.map((relay) =>
    isRelayDisabled(relay, { ...location, hostname: relay.hostname }, disabledLocation),
  );
  if (relaysDisabled.every((status) => status === DisabledReason.inactive)) {
    return DisabledReason.inactive;
  }

  const disabledDueToSelection = relaysDisabled.find(
    (status) => status === DisabledReason.entry || status === DisabledReason.exit,
  );

  if (
    relaysDisabled.every((status) => status !== undefined) &&
    disabledDueToSelection !== undefined
  ) {
    return disabledDueToSelection;
  }

  if (
    disabledLocation?.location &&
    compareRelayLocation(location, disabledLocation.location) &&
    city.relays.filter((relay) => relay.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}
