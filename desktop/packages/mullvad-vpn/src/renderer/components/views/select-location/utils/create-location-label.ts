import { sprintf } from 'sprintf-js';

import type { RelayLocation } from '../../../../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../../../../shared/gettext';
import { DisabledReason } from '../select-location-types';

export function createLocationLabel(
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
