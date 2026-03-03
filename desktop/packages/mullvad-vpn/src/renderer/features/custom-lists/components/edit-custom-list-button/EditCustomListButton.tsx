import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { CustomListLocation } from '../../../location/types';
import { EditCustomListDialog } from '../edit-custom-list-dialog';

export type EditCustomListButtonProps = IconButtonProps & {
  customList: CustomListLocation;
  loading?: boolean;
  onLoadingChange?: (loading: boolean) => void;
};

export function EditCustomListButton({
  customList,
  loading,
  onLoadingChange,
  ...props
}: EditCustomListButtonProps) {
  const [editCustomListDialogOpen, setEditCustomListDialogOpen] = React.useState(false);
  const showEditCustomListDialog = React.useCallback(() => {
    setEditCustomListDialogOpen(true);
  }, []);
  return (
    <>
      <IconButton
        variant="secondary"
        onClick={showEditCustomListDialog}
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
      <EditCustomListDialog
        customList={customList}
        open={editCustomListDialogOpen}
        onOpenChange={setEditCustomListDialogOpen}
        loading={loading}
        onLoadingChange={onLoadingChange}
      />
    </>
  );
}
