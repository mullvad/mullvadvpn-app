import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { useSelectLocationViewContext } from '../../../../components/views/select-location/SelectLocationViewContext';
import { Menu, type MenuProps } from '../../../../lib/components/menu';
import { CreateCustomListDialog } from '../../../custom-lists/components';
import { useCustomLists } from '../../../custom-lists/hooks';
import { useMultihop } from '../../../multihop/hooks';
import { DisabledReason, type GeographicalLocation, LocationType } from '../../types';
import {
  AddLocationToCustomListMenuOption,
  CreateCustomListMenuOption,
  SetAsEntryMenuOption,
  SetAsExitMenuOption,
} from './components';

export type GeographicalMenuProps = MenuProps & {
  location: GeographicalLocation;
};

export function GeographicalLocationMenu({
  onOpenChange,
  location,
  ...props
}: GeographicalMenuProps) {
  const { customLists } = useCustomLists();
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();
  const [createCustomListDialogOpen, setCreateCustomListDialogOpen] = React.useState(false);
  const handleOpenCreateCustomListDialog = React.useCallback(() => {
    setCreateCustomListDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  const showSetAsEntryMenuOption =
    !multihop ||
    (multihop &&
      locationType === LocationType.exit &&
      location.disabledReason !== DisabledReason.entry);

  const showSetAsExitMenuOption =
    multihop &&
    locationType === LocationType.entry &&
    location.disabledReason !== DisabledReason.exit;

  return (
    <>
      <Menu onOpenChange={onOpenChange} {...props}>
        <Menu.Popup>
          {showSetAsEntryMenuOption && <SetAsEntryMenuOption location={location} />}
          {showSetAsExitMenuOption && <SetAsExitMenuOption location={location} />}
          {(showSetAsEntryMenuOption || showSetAsExitMenuOption) && <Menu.Divider />}
          <Menu.Title>
            {sprintf(
              // TRANSLATORS: This is a label shown above a list of options related to custom lists.
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
