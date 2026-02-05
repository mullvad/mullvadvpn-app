import React from 'react';

import { compareRelayLocationGeographical } from '../../../../../../../../shared/daemon-rpc-types';
import log from '../../../../../../../../shared/logging';
import { useCustomLists } from '../../../../../../../features/location/hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { useLocationRowContext } from '../../LocationRowContext';

export type RemoveFromCustomListButtonProps = LocationListItemIconButtonProps;

export function RemoveFromCustomListButton(props: RemoveFromCustomListButtonProps) {
  const { source } = useLocationRowContext();
  const { customLists, updateCustomList } = useCustomLists();
  const onRemoveFromList = React.useCallback(async () => {
    if (source.location.customList) {
      // Find the list and remove the location from it.
      const list = customLists.find((list) => list.id === source.location.customList);
      if (list !== undefined) {
        const updatedList = {
          ...list,
          locations: list.locations.filter((location) => {
            return !compareRelayLocationGeographical(location, source.location);
          }),
        };

        try {
          await updateCustomList(updatedList);
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to edit custom list ${source.location.customList}: ${error.message}`);
        }
      }
    }
  }, [customLists, source.location, updateCustomList]);

  return (
    <>
      <LocationListItem.HeaderTrailingAction>
        <LocationListItem.IconButton onClick={onRemoveFromList} {...props}>
          <LocationListItem.IconButton.Icon icon="remove-circle" />
        </LocationListItem.IconButton>
      </LocationListItem.HeaderTrailingAction>
    </>
  );
}
