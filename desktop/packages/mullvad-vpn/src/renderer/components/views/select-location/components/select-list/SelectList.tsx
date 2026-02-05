import React from 'react';

import {
  compareRelayLocationGeographical,
  ICustomList,
  type RelayLocationGeographical,
} from '../../../../../../shared/daemon-rpc-types';
import { useCustomLists } from '../../../../../features/location/hooks';
import { IconButton } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { SelectableLabel } from '../../../../../lib/components/selectable-label';

interface SelectListProps {
  list: ICustomList;
  location: RelayLocationGeographical;
}

export function SelectList({ list, location }: SelectListProps) {
  const { addLocationToCustomList, removeLocationFromCustomList } = useCustomLists();
  const [loading, setLoading] = React.useState(false);

  // List should be disabled if location already is in list.
  const addedToList = list.locations.some((listLocation) =>
    compareRelayLocationGeographical(listLocation, location),
  );

  const handleClickAdd = React.useCallback(async () => {
    setLoading(true);
    await addLocationToCustomList(list.id, location);
    setLoading(false);
  }, [addLocationToCustomList, list.id, location]);

  const handleClickRemove = React.useCallback(async () => {
    setLoading(true);
    await removeLocationFromCustomList(list.id, location);
    setLoading(false);
  }, [list.id, location, removeLocationFromCustomList]);

  return (
    <ListItem position="solo">
      <ListItem.Item>
        <SelectableLabel selected={addedToList}>{list.name}</SelectableLabel>
        <ListItem.ActionGroup>
          {addedToList ? (
            <IconButton variant="secondary" disabled={loading} onClick={handleClickRemove}>
              <IconButton.Icon icon="remove-circle" />
            </IconButton>
          ) : (
            <IconButton variant="secondary" disabled={loading} onClick={handleClickAdd}>
              <IconButton.Icon icon="add-circle" />
            </IconButton>
          )}
        </ListItem.ActionGroup>
      </ListItem.Item>
    </ListItem>
  );
}
