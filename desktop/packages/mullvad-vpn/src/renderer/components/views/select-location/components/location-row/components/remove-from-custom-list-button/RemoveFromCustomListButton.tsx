import React from 'react';

import { useCustomLists } from '../../../../../../../features/location/hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { useLocationRowContext } from '../../LocationRowContext';

export type RemoveFromCustomListButtonProps = LocationListItemIconButtonProps & {};

export function RemoveFromCustomListButton(props: RemoveFromCustomListButtonProps) {
  const {
    source: { location },
  } = useLocationRowContext();
  const { removeLocationFromCustomList } = useCustomLists();
  const [loading, setLoading] = React.useState(false);

  const onRemoveFromList = React.useCallback(async () => {
    setLoading(true);
    if ('customList' in location && 'country' in location && location.customList) {
      await removeLocationFromCustomList(location.customList, location);
    }
    setLoading(false);
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
