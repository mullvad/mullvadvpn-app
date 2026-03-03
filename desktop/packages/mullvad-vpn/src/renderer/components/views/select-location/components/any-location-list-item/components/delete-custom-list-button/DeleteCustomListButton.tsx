import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import type { CustomListLocation } from '../../../../../../../features/location/types';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { ConfirmDeleteCustomListDialog } from '../../../confirm-delete-custom-list-dialog';

export type DeleteCustomListButtonProps = LocationListItemIconButtonProps & {
  customList: CustomListLocation;
  loading?: boolean;
  onLoadingChange?: (loading: boolean) => void;
};

export function DeleteCustomListButton({
  customList,
  loading,
  onLoadingChange,
  ...props
}: DeleteCustomListButtonProps) {
  const [deleteCustomListDialogOpen, setDeleteCustomListDialogOpen] = React.useState(false);
  const showDeleteCustomListDialog = React.useCallback(() => {
    setDeleteCustomListDialogOpen(true);
  }, []);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton
          onClick={showDeleteCustomListDialog}
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
      <ConfirmDeleteCustomListDialog
        customList={customList}
        open={deleteCustomListDialogOpen}
        onOpenChange={setDeleteCustomListDialogOpen}
        loading={loading}
        onLoadingChange={onLoadingChange}
      />
    </>
  );
}
