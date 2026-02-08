import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
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
        <LocationListItem.IconButton
          onClick={show}
          aria-label={sprintf(
            // TRANSLATORS: Accessibility label for button to add a location to a custom list.
            // TRANSLATORS: The placeholder is replaced with the name of the location.
            messages.pgettext('accessibility', 'Add %(locationName)s to custom list'),
            {
              locationName: location.label,
            },
          )}
          {...props}>
          <LocationListItem.IconButton.Icon icon="add-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>

      <AddToListDialog open={open} onOpenChange={setOpen} location={location} />
    </>
  );
}
