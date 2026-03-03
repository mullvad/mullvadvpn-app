import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { CustomListLocation } from '../../../locations/types';
import { DeleteCustomListDialog } from '../delete-custom-list-dialog';

export type DeleteCustomListButtonProps = IconButtonProps & {
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
      <IconButton
        onClick={showDeleteCustomListDialog}
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
      <DeleteCustomListDialog
        customList={customList}
        open={deleteCustomListDialogOpen}
        onOpenChange={setDeleteCustomListDialogOpen}
        loading={loading}
        onLoadingChange={onLoadingChange}
      />
    </>
  );
}
