import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { CustomListLocation } from '../../../../select-location-types';
import { EditListDialog } from '../../../edit-list-dialog';

export type EditCustomListButtonProps = LocationListItemIconButtonProps & {
  customList: CustomListLocation;
};

export function EditCustomListButton({ customList, ...props }: EditCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);

  const handleOnClick = React.useCallback(() => {
    setOpen(true);
  }, []);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton
          onClick={handleOnClick}
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
          <LocationListItem.IconButton.Icon icon="edit-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <EditListDialog customList={customList} open={open} onOpenChange={setOpen} />
    </>
  );
}
