import {
  compareRelayLocation,
  RelayLocation,
  RelayLocationCustomList,
} from '../../../../../shared/daemon-rpc-types';
import { DisabledReason, type GeographicalLocation } from '../select-location-types';

export function isCustomListDisabled(
  location: RelayLocationCustomList,
  locations: GeographicalLocation[],
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
) {
  const locationsDisabled = locations.map((location) => location.disabledReason);
  if (locationsDisabled.every((status) => status === DisabledReason.inactive)) {
    return DisabledReason.inactive;
  }

  const disabledDueToSelection = locationsDisabled.find(
    (status) => status === DisabledReason.entry || status === DisabledReason.exit,
  );
  if (
    locationsDisabled.every((status) => status !== undefined) &&
    disabledDueToSelection !== undefined
  ) {
    return disabledDueToSelection;
  }

  if (
    disabledLocation &&
    compareRelayLocation(location, disabledLocation.location) &&
    locations.filter((location) => location.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}
