import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../lib/components';
import type { GeographicalLocation } from '../../../locations/types';
import { useGetCustomListById, useRemoveLocationFromCustomList } from '../../hooks';

export type RemoveFromCustomListButtonProps = IconButtonProps & {
  location: GeographicalLocation;
  loading?: boolean;
  onLoadingChange?: (loading: boolean) => void;
};

export function RemoveLocationFromCustomListButton({
  location,
  loading,
  onLoadingChange,
  ...props
}: RemoveFromCustomListButtonProps) {
  const removeLocationFromCustomList = useRemoveLocationFromCustomList();
  const getCustomListById = useGetCustomListById();

  const handleOnClick = React.useCallback(async () => {
    const customListId = location.details.customList;
    if (customListId !== undefined) {
      onLoadingChange?.(true);
      const success = await removeLocationFromCustomList(customListId, location.details);

      // Only set loading to false if failed to keep disabled state while animating out
      if (!success) {
        onLoadingChange?.(false);
      }
    }
  }, [location.details, removeLocationFromCustomList, onLoadingChange]);

  const customListId = location.details.customList;
  const customList = getCustomListById(customListId ?? '');
  return (
    <IconButton
      variant="secondary"
      onClick={handleOnClick}
      disabled={loading || customList === undefined}
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for button to remove a location from a custom list.
        // TRANSLATORS: The first placeholder is replaced with the name of the location.
        // TRANSLATORS: The second placeholder is replaced with the name of the custom list.
        messages.pgettext('accessibility', 'Remove %(location)s from %(customList)s'),
        {
          location: location.label,
          customList: customList?.name,
        },
      )}
      {...props}>
      <IconButton.Icon icon="remove-circle" />
    </IconButton>
  );
}
