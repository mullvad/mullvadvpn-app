import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { GeographicalLocation } from '../../../locations/types';
import { useCustomLists } from '../../hooks';
import { AddLocationToCustomListDialog } from '../add-location-to-custom-list-dialog';

export type AddToCustomListButtonProps = IconButtonProps & {
  location: GeographicalLocation;
};

export function AddLocationToCustomListButton({ location, ...props }: AddToCustomListButtonProps) {
  const [open, setOpen] = React.useState(false);
  const handleOpenDialog = React.useCallback(() => setOpen(true), []);
  const { customLists } = useCustomLists();
  const disabled = customLists.length === 0;

  return (
    <>
      <IconButton
        onClick={handleOpenDialog}
        variant="secondary"
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
        <IconButton.Icon icon="add-circle" />
      </IconButton>

      <AddLocationToCustomListDialog open={open} onOpenChange={setOpen} location={location} />
    </>
  );
}
