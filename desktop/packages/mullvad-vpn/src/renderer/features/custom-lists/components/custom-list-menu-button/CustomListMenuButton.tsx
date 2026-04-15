import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { CustomListLocation } from '../../../locations/types';

export type CustomListMenuButtonProps = IconButtonProps & {
  customList: CustomListLocation;
};

export function CustomListMenuButton({ customList, ...props }: CustomListMenuButtonProps) {
  return (
    <IconButton
      variant="secondary"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to open a custom list menu.
        // TRANSLATORS: The placeholder is replaced with the name of the custom list.
        messages.pgettext('accessibility', 'Open menu for %(listName)s'),
        {
          listName: customList.label,
        },
      )}
      {...props}>
      <IconButton.Icon icon="more-horizontal" />
    </IconButton>
  );
}
