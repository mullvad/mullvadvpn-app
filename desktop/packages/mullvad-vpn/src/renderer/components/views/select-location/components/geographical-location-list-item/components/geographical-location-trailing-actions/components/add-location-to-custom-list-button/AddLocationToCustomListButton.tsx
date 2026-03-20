import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../../../../../features/custom-lists/hooks';
import type { GeographicalLocation } from '../../../../../../../../../features/locations/types';
import { IconButton, type IconButtonProps } from '../../../../../../../../../lib/components';

export type AddToCustomListButtonProps = IconButtonProps & {
  location: GeographicalLocation;
};

export function AddLocationToCustomListButton({ location, ...props }: AddToCustomListButtonProps) {
  const { customLists } = useCustomLists();
  const disabled = customLists.length === 0;

  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to add a location to a custom list.
        // TRANSLATORS: The placeholder is replaced with the name of the location.
        messages.pgettext('accessibility', 'Add %(locationName)s to custom list'),
        {
          locationName: location.label,
        },
      )}
      disabled={disabled}
      {...props}>
      <IconButton.Icon icon="add-circle" />
    </IconButton>
  );
}
