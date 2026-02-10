import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { CustomListLocation } from '../../../../select-location-types';
import { ConfirmDeleteCustomListDialog } from '../../../confirm-delete-custom-list-dialog';

export type DeleteCustomListButtonProps = LocationListItemIconButtonProps & {
  customList: CustomListLocation;
};

export function DeleteCustomListButton({ customList, ...props }: DeleteCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);

  const show = React.useCallback(() => {
    setOpen(true);
  }, []);

  const hide = React.useCallback(() => {
    setOpen(false);
  }, []);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton
          onClick={show}
          aria-label={sprintf(
            // TRANSLATORS: Accessibility label for button to delete a custom list.
            // TRANSLATORS: The placeholder is replaced with the name of the custom list.
            messages.pgettext('accessibility', 'Delete custom list %(listName)s'),
            {
              listName: customList.label,
            },
          )}
          {...props}>
          <LocationListItem.IconButton.Icon icon="cross-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <ConfirmDeleteCustomListDialog customList={customList} open={open} onOpenChange={hide} />
    </>
  );
}
