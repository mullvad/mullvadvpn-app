import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { GeographicalLocation } from '../../types';

export type GeographicalLocationMenuButtonProps = IconButtonProps & {
  location: GeographicalLocation;
};

export function GeographicalLocationMenuButton({
  location,
  ...props
}: GeographicalLocationMenuButtonProps) {
  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to edit a geographical location.
        // TRANSLATORS: The placeholder is replaced with the name of the geographical location.
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
