import { sprintf } from 'sprintf-js';

import {
  compareRelayLocation,
  compareRelayLocationLoose,
  LiftedConstraint,
  RelayLocation,
  RelayLocationCity,
  RelayLocationCountry,
  RelayLocationCustomList,
  RelayLocationRelay,
} from '../../../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../../../shared/gettext';
import {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import { DisabledReason, type GeographicalLocation } from './select-location-types';

export function isSelected(
  relayLocation: RelayLocation,
  selected?: LiftedConstraint<RelayLocation>,
) {
  return selected !== 'any' && compareRelayLocationLoose(selected, relayLocation);
}

// Formats the label that is discplayed for a country, city or relay
export function formatRowName(
  name: string,
  location: RelayLocation,
  disabledReason?: DisabledReason,
): string {
  const translatedName = 'hostname' in location ? name : relayLocations.gettext(name);

  // In some situations the exit/entry server should be marked on a location
  let info: string | undefined;
  if (disabledReason === DisabledReason.entry) {
    info = messages.pgettext('select-location-view', 'Entry');
  } else if (disabledReason === DisabledReason.exit) {
    info = messages.pgettext('select-location-view', 'Exit');
  }

  return info !== undefined
    ? sprintf(
        // TRANSLATORS: This is used for appending information about a location.
        // TRANSLATORS: E.g. "Gothenburg (Entry)" if Gothenburg has been selected as the entrypoint.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(location)s - Translated location name
        // TRANSLATORS: %(info)s - Information about the location
        messages.pgettext('select-location-view', '%(location)s (%(info)s)'),
        {
          location: translatedName,
          info,
        },
      )
    : translatedName;
}

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

export function isCityDisabled(
  city: IRelayLocationCityRedux,
  location: RelayLocationCity,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
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
    disabledLocation &&
    compareRelayLocation(location, disabledLocation.location) &&
    city.relays.filter((relay) => relay.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}

export function isCountryDisabled(
  country: IRelayLocationCountryRedux,
  location: RelayLocationCountry,
  disabledLocation?: { location: RelayLocation; reason: DisabledReason },
): DisabledReason | undefined {
  const citiesDisabled = country.cities.map((city) =>
    isCityDisabled(city, { ...location, city: city.code }, disabledLocation),
  );
  if (citiesDisabled.every((status) => status === DisabledReason.inactive)) {
    return DisabledReason.inactive;
  }

  const disabledDueToSelection = citiesDisabled.find(
    (status) => status === DisabledReason.entry || status === DisabledReason.exit,
  );
  if (
    citiesDisabled.every((status) => status !== undefined) &&
    disabledDueToSelection !== undefined
  ) {
    return disabledDueToSelection;
  }

  if (
    disabledLocation &&
    compareRelayLocation(location, disabledLocation.location) &&
    country.cities.flatMap((city) => city.relays).filter((relay) => relay.active).length <= 1
  ) {
    return disabledLocation.reason;
  }

  return undefined;
}

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
