import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../../shared/gettext';
import type { CustomListLocation } from '../../../../../../../../../features/locations/types';
import { IconButton, type IconButtonProps } from '../../../../../../../../../lib/components';

export type EditCustomListButtonProps = IconButtonProps & {
  customList: CustomListLocation;
};

export function EditCustomListButton({ customList, ...props }: EditCustomListButtonProps) {
  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to edit a custom list.
        // TRANSLATORS: The placeholder is replaced with the name of the custom list.
        messages.pgettext('accessibility', 'Edit custom list %(listName)s'),
        {
          listName: customList.label,
        },
      )}
      {...props}>
      <IconButton.Icon icon="edit-circle" />
    </IconButton>
  );
}
