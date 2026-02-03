import { useCallback } from 'react';

import {
  compareRelayLocationGeographical,
  ICustomList,
  RelayLocation,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { IconButton } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';

interface SelectListProps {
  list: ICustomList;
  location: RelayLocation;
  add: (list: ICustomList) => void;
}

export function SelectList(props: SelectListProps) {
  const { add } = props;

  const onAdd = useCallback(() => add(props.list), [add, props.list]);

  // List should be disabled if location already is in list.
  const disabled = props.list.locations.some((location) =>
    compareRelayLocationGeographical(location, props.location),
  );

  return (
    <ListItem onClick={onAdd} disabled={disabled} position="solo">
      <ListItem.Item>
        <ListItem.Label>
          {props.list.name} {disabled && messages.pgettext('select-location-view', '(Added)')}
        </ListItem.Label>
        <ListItem.ActionGroup>
          <IconButton variant="secondary">
            <IconButton.Icon icon="add-circle" />
          </IconButton>
        </ListItem.ActionGroup>
      </ListItem.Item>
    </ListItem>
  );
}
