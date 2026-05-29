import React from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import { useMenuContext } from '../../../../../../lib/components/menu/MenuContext';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../../../../daita/hooks';
import { useMultihop } from '../../../../../multihop/hooks';
import { useRelayLocations, useSelectedLocations } from '../../../../hooks';
import type { GeographicalLocation } from '../../../../types';
import { isLocationSelected } from '../../../../utils';

export type SetAsEntryMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
};

export function SetAsEntryMenuOption({ location, ...props }: SetAsEntryMenuOptionProps) {
  const { multihop, setMultihop } = useMultihop();
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { selectEntryRelayLocation } = useRelayLocations();
  const { onOpenChange } = useMenuContext();
  const { entry, exit } = useSelectedLocations();
  const isExitSelected = isLocationSelected(location.details, exit);

  const handleClick = React.useCallback(async () => {
    if (!multihop) {
      await setMultihop({ enabled: true, entryLocation: location.details });
    }
    if (multihop) {
      if (isExitSelected) {
        // Swap entry and exit location
        await setMultihop({ enabled: true, entryLocation: location.details, exitLocation: entry });
      } else {
        await selectEntryRelayLocation(location.details);
      }
    }
    onOpenChange?.(false);
  }, [
    multihop,
    onOpenChange,
    setMultihop,
    location.details,
    isExitSelected,
    entry,
    selectEntryRelayLocation,
  ]);

  const disabled = (daitaEnabled && !daitaDirectOnly) || (!multihop && isExitSelected);
  const label = disabled
    ? // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Text for button that sets a location as entry relay
      messages.gettext('Set as multihop entry (incompatible)')
    : // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Text for button that sets a location as entry relay when button is disabled.
      messages.gettext('Set as multihop entry');

  return (
    <Menu.Option disabled={disabled} {...props}>
      <Menu.Option.Trigger onClick={handleClick}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="location-add" />
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Text for button that sets a location as entry relay
              label
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
