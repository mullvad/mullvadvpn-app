import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import type { AnyLocation } from '../../../../features/locations/types';
import { useMultihop } from '../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useLocationAriaLabel(location: AnyLocation) {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  if (multihop !== 'always') {
    if (location.needsOtherEntry) {
      return sprintf(
        // TRANSLATORS: Accessibility label for button that connects to a location and uses an automatic entry.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
        messages.pgettext(
          'accessibility',
          'Connect to %(location)s, will connect via an automatic entry',
        ),
        {
          location: location.label,
        },
      );
    } else {
      return sprintf(
        // TRANSLATORS: Accessibility label for button that connects to a location.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
        messages.pgettext('accessibility', 'Connect to %(location)s'),
        {
          location: location.label,
        },
      );
    }
  }
  if (locationType === 'entry') {
    return sprintf(
      // TRANSLATORS: Accessibility label for button that sets an entry location.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
      messages.pgettext('accessibility', 'Use %(location)s as entry'),
      {
        location: location.label,
      },
    );
  } else {
    return sprintf(
      // TRANSLATORS: Accessibility label for button that connects and sets an exit location.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
      messages.pgettext('accessibility', 'Connect and use %(location)s as exit'),
      {
        location: location.label,
      },
    );
  }
}
