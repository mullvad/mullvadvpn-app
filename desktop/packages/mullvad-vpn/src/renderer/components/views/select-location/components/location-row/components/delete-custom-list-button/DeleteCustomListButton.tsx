import React from 'react';

import log from '../../../../../../../../shared/logging';
import { useCustomLists } from '../../../../../../../features/location/hooks';
import { useBoolean } from '../../../../../../../lib/utility-hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { DeleteConfirmDialog } from '../../../delete-confirm-dialog';
import { useLocationRowContext } from '../../LocationRowContext';

export type DeleteCustomListButtonProps = LocationListItemIconButtonProps;

export function DeleteCustomListButton(props: DeleteCustomListButtonProps) {
  const [deleteDialogVisible, showDeleteDialog, hideDeleteDialog] = useBoolean();
  const { source } = useLocationRowContext();
  const { deleteCustomList } = useCustomLists();

  const confirmDeleteCustomList = React.useCallback(async () => {
    if (source.location.customList) {
      try {
        await deleteCustomList(source.location.customList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to delete custom list ${source.location.customList}: ${error.message}`);
      }
    }
  }, [deleteCustomList, source.location.customList]);

  return (
    <>
      <LocationListItem.IconButton onClick={showDeleteDialog} {...props}>
        <LocationListItem.IconButton.Icon icon="cross-circle" />
      </LocationListItem.IconButton>

      {'list' in source && (
        <DeleteConfirmDialog
          list={source.list}
          isOpen={deleteDialogVisible}
          hide={hideDeleteDialog}
          confirm={confirmDeleteCustomList}
        />
      )}
    </>
  );
}
