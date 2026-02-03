import React from 'react';

import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { AddToListDialog } from '../../../add-to-list-dialog';
import { useLocationRowContext } from '../../LocationRowContext';

export type AddToCustomListButtonProps = LocationListItemIconButtonProps;

export function AddToCustomListButton(props: AddToCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);
  const show = React.useCallback(() => setOpen(true), []);
  const {
    source: { location },
  } = useLocationRowContext();

  return (
    <>
      <LocationListItem.IconButton onClick={show} {...props}>
        <LocationListItem.IconButton.Icon icon="add-circle" />
      </LocationListItem.IconButton>

      {'country' in location && (
        <AddToListDialog open={open} onOpenChange={setOpen} location={location} />
      )}
    </>
  );
}
