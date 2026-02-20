import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../../../features/location/hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { GeographicalLocation } from '../../../../select-location-types';
import { AddToCustomListDialog } from '../../../add-to-custom-list-dialog';

export type AddToCustomListButtonProps = LocationListItemIconButtonProps & {
  location: GeographicalLocation;
};

export function AddToCustomListButton({ location, ...props }: AddToCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);
  const handleOpenDialog = React.useCallback(() => setOpen(true), []);
  const { customLists } = useCustomLists();
  const disabled = customLists.length === 0;

  return (
    <>
      <LocationListItem.IconButton
        onClick={handleOpenDialog}
        aria-label={sprintf(
          // TRANSLATORS: Accessibility label for button to add a location to a custom list.
          // TRANSLATORS: The placeholder is replaced with the name of the location.
          messages.pgettext('accessibility', 'Add %(locationName)s to custom list'),
          {
            locationName: location.label,
          },
        )}
        disabled={disabled}
        {...props}>
        <LocationListItem.IconButton.Icon icon="add-circle" />
      </LocationListItem.IconButton>

      <AddToCustomListDialog open={open} onOpenChange={setOpen} location={location} />
    </>
  );
}
