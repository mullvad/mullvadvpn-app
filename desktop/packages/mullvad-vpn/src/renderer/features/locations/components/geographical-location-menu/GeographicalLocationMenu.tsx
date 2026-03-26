import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Menu, type MenuProps } from '../../../../lib/components/menu';
import { AddLocationToCustomListDialog } from '../../../custom-lists/components';
import type { GeographicalLocation } from '../../types';

export type GeographicalMenuProps = MenuProps & {
  location: GeographicalLocation;
};

export function GeographicalLocationMenu({
  onOpenChange,
  location,
  ...props
}: GeographicalMenuProps) {
  const [addLocationToCustomListDialogOpen, setAddLocationToCustomListDialogOpen] =
    React.useState(false);

  const showAddToCustomListDialog = React.useCallback(
    () => setAddLocationToCustomListDialogOpen(true),
    [],
  );

  return (
    <>
      <Menu onOpenChange={onOpenChange} {...props}>
        <Menu.Popup>
          <Menu.Option>
            <Menu.Option.Trigger onClick={showAddToCustomListDialog}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="add-circle" />
                <Menu.Option.Item.Label>
                  {messages.gettext('Add to custom list')}
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
        </Menu.Popup>
      </Menu>
      <AddLocationToCustomListDialog
        open={addLocationToCustomListDialogOpen}
        onOpenChange={setAddLocationToCustomListDialogOpen}
        location={location}
      />
    </>
  );
}
