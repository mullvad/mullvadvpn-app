import React from 'react';

import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { GeographicalLocation } from '../../../../select-location-types';
import { AddToListDialog } from '../../../add-to-list-dialog';

export type AddToCustomListButtonProps = LocationListItemIconButtonProps & {
  location: GeographicalLocation;
};

export function AddToCustomListButton({ location, ...props }: AddToCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);
  const show = React.useCallback(() => setOpen(true), []);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton onClick={show} {...props}>
          <LocationListItem.IconButton.Icon icon="add-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <AddToListDialog open={open} onOpenChange={setOpen} location={location} />
    </>
  );
}
