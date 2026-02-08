import React from 'react';

import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { CustomListLocation } from '../../../../select-location-types';
import { DeleteConfirmDialog } from '../../../delete-confirm-dialog';

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
        <LocationListItem.IconButton onClick={show} {...props}>
          <LocationListItem.IconButton.Icon icon="cross-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <DeleteConfirmDialog customList={customList} open={open} onOpenChange={hide} />
    </>
  );
}
