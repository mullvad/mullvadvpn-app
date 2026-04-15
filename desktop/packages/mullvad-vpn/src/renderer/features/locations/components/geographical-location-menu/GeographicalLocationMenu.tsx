import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Menu, type MenuProps } from '../../../../lib/components/menu';
import { CreateCustomListDialog } from '../../../custom-lists/components';
import { useCustomLists } from '../../../custom-lists/hooks';
import type { GeographicalLocation } from '../../types';
import { AddLocationToCustomListMenuOption } from './components';
import { CreateCustomListMenuOption } from './components/create-custom-list-menu-option';

export type GeographicalMenuProps = MenuProps & {
  location: GeographicalLocation;
};

export function GeographicalLocationMenu({
  onOpenChange,
  location,
  ...props
}: GeographicalMenuProps) {
  const { customLists } = useCustomLists();
  const [createCustomListDialogOpen, setCreateCustomListDialogOpen] = React.useState(false);
  const handleOpenCreateCustomListDialog = React.useCallback(() => {
    setCreateCustomListDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  return (
    <>
      <Menu onOpenChange={onOpenChange} {...props}>
        <Menu.Popup>
          <Menu.Title>
            {sprintf(
              // TRANSLATORS: This is a label shown above a list of options.
              // TRANSLATORS: Available placeholder:
              // TRANSLATORS: %(locationName)s - The name of the location being added to the list.
              messages.pgettext('custom-list-feature', 'Add %(locationName)s to list'),
              {
                locationName: location.label,
              },
            )}
          </Menu.Title>
          {customLists.map((customList) => (
            <AddLocationToCustomListMenuOption
              key={customList.id}
              location={location}
              customList={customList}
            />
          ))}
          <CreateCustomListMenuOption
            location={location}
            onClick={handleOpenCreateCustomListDialog}
          />
        </Menu.Popup>
      </Menu>
      <CreateCustomListDialog
        location={location}
        open={createCustomListDialogOpen}
        onOpenChange={setCreateCustomListDialogOpen}
      />
    </>
  );
}
