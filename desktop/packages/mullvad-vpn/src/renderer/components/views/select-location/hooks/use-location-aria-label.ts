import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { LocationType } from '../../../../features/locations/types';
import { useMultihop } from '../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useLocationAriaLabel(location: string) {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  if (multihop !== 'always') {
    return sprintf(
      // TRANSLATORS: Accessibility label for button that connects to a location.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
      messages.pgettext('accessibility', 'Connect to %(location)s'),
      {
        location: location,
      },
    );
  }
  if (locationType === LocationType.entry) {
    return sprintf(
      // TRANSLATORS: Accessibility label for button that sets an entry location.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
      messages.pgettext('accessibility', 'Use %(location)s as entry'),
      {
        location: location,
      },
    );
  } else {
    return sprintf(
      // TRANSLATORS: Accessibility label for button that connects and sets an exit location.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
      messages.pgettext('accessibility', 'Connect and use %(location)s as exit'),
      {
        location: location,
      },
    );
  }
}
