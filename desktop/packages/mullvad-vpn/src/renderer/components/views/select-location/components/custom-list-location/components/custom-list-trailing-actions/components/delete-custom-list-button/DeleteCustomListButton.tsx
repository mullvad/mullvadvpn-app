import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../../shared/gettext';
import type { CustomListLocation } from '../../../../../../../../../features/locations/types';
import { IconButton, type IconButtonProps } from '../../../../../../../../../lib/components';

export type DeleteCustomListButtonProps = IconButtonProps & {
  customList: CustomListLocation;
};

export function DeleteCustomListButton({ customList, ...props }: DeleteCustomListButtonProps) {
  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to delete a custom list.
        // TRANSLATORS: The placeholder is replaced with the name of the custom list.
        messages.pgettext('accessibility', 'Delete custom list %(listName)s'),
        {
          listName: customList.label,
        },
      )}
      {...props}>
      <IconButton.Icon icon="cross-circle" />
    </IconButton>
  );
}
