import React from 'react';
import { sprintf } from 'sprintf-js';

import {
  compareRelayLocationGeographical,
  ICustomList,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../features/location/hooks';
import { IconButton } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { SelectableLabel } from '../../../../../lib/components/selectable-label';
import type { GeographicalLocation } from '../../select-location-types';

interface SelectListProps {
  list: ICustomList;
  location: GeographicalLocation;
}

export function SelectList({ list, location }: SelectListProps) {
  const { addLocationToCustomList } = useCustomLists();
  const [loading, setLoading] = React.useState(false);

  // List should be disabled if location already is in list.
  const addedToList = list.locations.some((listLocation) =>
    compareRelayLocationGeographical(listLocation, location.details),
  );

  const handleClickAdd = React.useCallback(async () => {
    setLoading(true);
    await addLocationToCustomList(list.id, location.details);
    setLoading(false);
  }, [addLocationToCustomList, list.id, location]);

  return (
    <ListItem position="solo">
      <ListItem.Item>
        <SelectableLabel selected={addedToList}>{list.name}</SelectableLabel>
        <ListItem.ActionGroup>
          <IconButton
            variant="secondary"
            disabled={loading || addedToList}
            onClick={handleClickAdd}
            aria-label={sprintf(
              messages.pgettext('accessibility', 'Add %(location)s to %(listName)s'),
              {
                location: location.label,
                listName: list.name,
              },
            )}>
            <IconButton.Icon icon="add-circle" />
          </IconButton>
        </ListItem.ActionGroup>
      </ListItem.Item>
    </ListItem>
  );
}
