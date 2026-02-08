import React from 'react';

import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { CustomListLocation } from '../../../../select-location-types';
import { EditListDialog } from '../../../edit-list-dialog';

export type EditCustomListButtonProps = LocationListItemIconButtonProps & {
  customList: CustomListLocation;
};

export function EditCustomListButton({ customList, ...props }: EditCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);

  const handleOpen = React.useCallback(() => {
    setOpen(true);
  }, []);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton onClick={handleOpen} variant="secondary" {...props}>
          <LocationListItem.IconButton.Icon icon="edit-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <EditListDialog customList={customList} open={open} onOpenChange={setOpen} />
    </>
  );
}
