import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../../../features/location/hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { GeographicalLocation } from '../../../../select-location-types';
import { useGeographicalLocationListItemContext } from '../../../geographical-location-list-item/GeographicalLocationListItemContext';

export type RemoveFromCustomListButtonProps = LocationListItemIconButtonProps & {
  location: GeographicalLocation;
};

export function RemoveFromCustomListButton({
  location,
  ...props
}: RemoveFromCustomListButtonProps) {
  const { removeLocationFromCustomList, getCustomListById } = useCustomLists();
  const { loading, setLoading } = useGeographicalLocationListItemContext();

  const customList = getCustomListById(location.details.customList);

  const handleOnClick = React.useCallback(async () => {
    const customList = location.details.customList;
    if (customList !== undefined) {
      setLoading(true);
      const success = await removeLocationFromCustomList(customList, location.details);

      // Only set loading to false if failed to keep disabled state while animating out
      if (!success) {
        setLoading(false);
      }
    }
  }, [location.details, removeLocationFromCustomList, setLoading]);

  return (
    <LocationListItem.IconButton
      onClick={handleOnClick}
      disabled={loading}
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
      <LocationListItem.IconButton.Icon icon="remove-circle" />
    </LocationListItem.IconButton>
  );
}
