import { messages } from '../../../../../../shared/gettext';
import type { GeographicalLocation } from '../../../../location/types';

export function useLocationTypeMessage(location: GeographicalLocation) {
  if (location.type === 'relay') {
    return (
      // TRANSLATORS: Refers to our VPN relays/servers, used in a message when adding a relay to a custom list.
      messages.pgettext('select-location-view', 'Relay')
    );
  } else if (location.type === 'city') {
    return (
      // TRANSLATORS: Refers to cities, used in a message when adding a city to a custom list.
      messages.pgettext('select-location-view', 'City')
    );
  } else {
    return (
      // TRANSLATORS: Refers to countries, used in a message when adding a country to a custom list.
      messages.pgettext('select-location-view', 'Country')
    );
  }
}
