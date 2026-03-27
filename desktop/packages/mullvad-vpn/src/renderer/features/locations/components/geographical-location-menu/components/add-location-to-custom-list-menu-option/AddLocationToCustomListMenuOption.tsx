import React from 'react';
import { sprintf } from 'sprintf-js';

import {
  compareRelayLocationGeographical,
  type ICustomList,
} from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import { useMenuContext } from '../../../../../../lib/components/menu/MenuContext';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import { useAddLocationToCustomList } from '../../../../../custom-lists/hooks';
import type { GeographicalLocation } from '../../../../types';

export type AddLocationToCustomListMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
  customList: ICustomList;
};

export function AddLocationToCustomListMenuOption({
  location,
  customList,
  ...props
}: AddLocationToCustomListMenuOptionProps) {
  const addLocationToCustomList = useAddLocationToCustomList();
  const { onOpenChange } = useMenuContext();

  const addedToList = customList.locations.some((listLocation) =>
    compareRelayLocationGeographical(listLocation, location.details),
  );

  const handleClickAdd = React.useCallback(async () => {
    await addLocationToCustomList(customList.id, location.details);
    onOpenChange?.(false);
  }, [addLocationToCustomList, customList.id, location, onOpenChange]);

  return (
    <Menu.Option disabled={addedToList} {...props}>
      <Menu.Option.Trigger
        onClick={handleClickAdd}
        aria-label={sprintf(
          // TRANSLATORS: This is an accessibility label for a button that adds a location to a custom list.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(location)s - The name of the location being added to the list.
          // TRANSLATORS: %(listName)s - The name of the custom list the location will be added to.
          messages.pgettext('accessibility', 'Add %(location)s to %(listName)s'),
          {
            location: location.label,
            listName: customList.name,
          },
        )}>
        <Menu.Option.Item>
          <Menu.Option.Item.Label>
            {addedToList
              ? sprintf(
                  // TRANSLATORS: Label for disabled menu option when a location has already been added to a custom list.
                  // TRANSLATORS: Available placeholders:
                  // TRANSLATORS: %(customList)s - The name of the custom list the location has been added to.
                  messages.pgettext('custom-list-feature', '%(customList)s (Added)'),
                  {
                    customList: customList.name,
                  },
                )
              : customList.name}
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
