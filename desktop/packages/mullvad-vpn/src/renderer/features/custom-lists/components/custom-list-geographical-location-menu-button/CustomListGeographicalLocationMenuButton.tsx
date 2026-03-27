import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { GeographicalLocation } from '../../../locations/types';

export type CustomListGeographicalLocationMenuButtonProps = IconButtonProps & {
  location: GeographicalLocation;
};

export function CustomListGeographicalLocationMenuButton({
  location,
  ...props
}: CustomListGeographicalLocationMenuButtonProps) {
  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to open a location menu.
        // TRANSLATORS: The placeholder is replaced with the name of the location.
        messages.pgettext('accessibility', 'Open menu for %(locationName)s'),
        {
          locationName: location.label,
        },
      )}
      {...props}>
      <IconButton.Icon icon="more-horizontal" />
    </IconButton>
  );
}
