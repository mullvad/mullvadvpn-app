import React from 'react';

import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { EditListDialog } from '../../../edit-list-dialog';
import { useLocationRowContext } from '../../LocationRowContext';

export type EditCustomListButtonProps = LocationListItemIconButtonProps;

export function EditCustomListButton(props: EditCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);
  const { source } = useLocationRowContext();

  const handleOpen = React.useCallback(() => {
    setOpen(true);
  }, []);

  return (
    <>
      <LocationListItem.IconButton onClick={handleOpen} variant="secondary" {...props}>
        <LocationListItem.IconButton.Icon icon="edit-circle" />
      </LocationListItem.IconButton>

      {'list' in source && <EditListDialog open={open} onOpenChange={setOpen} />}
    </>
  );
}
