import React from 'react';

import { useCustomLists } from '../../../../../../../features/location/hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import type { GeographicalLocation } from '../../../../select-location-types';

export type RemoveFromCustomListButtonProps = LocationListItemIconButtonProps & {
  location: GeographicalLocation;
};

export function RemoveFromCustomListButton({
  location,
  ...props
}: RemoveFromCustomListButtonProps) {
  const { removeLocationFromCustomList } = useCustomLists();
  const [loading, setLoading] = React.useState(false);

  const onRemoveFromList = React.useCallback(async () => {
    const customList = location.details.customList;
    if (customList !== undefined) {
      setLoading(true);
      await removeLocationFromCustomList(customList, location.details);
      setLoading(false);
    }
  }, [location, removeLocationFromCustomList]);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton onClick={onRemoveFromList} disabled={loading} {...props}>
          <LocationListItem.IconButton.Icon icon="remove-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>
    </>
  );
}
