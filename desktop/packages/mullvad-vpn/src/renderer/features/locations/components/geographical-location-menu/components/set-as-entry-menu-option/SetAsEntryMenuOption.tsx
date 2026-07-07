import React from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import { useMenuContext } from '../../../../../../lib/components/menu/MenuContext';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import { useIsDaitaEnabledWithoutDirectOnly } from '../../../../../daita/hooks';
import { useMultihop } from '../../../../../multihop/hooks';
import { useRelayLocations, useSelectedLocations } from '../../../../hooks';
import type { GeographicalLocation } from '../../../../types';
import { isLocationSelected } from '../../../../utils';

export type SetAsEntryMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
};

export function SetAsEntryMenuOption({ location, ...props }: SetAsEntryMenuOptionProps) {
  const { multihop, setMultihop } = useMultihop();
  const { selectEntryRelayLocation } = useRelayLocations();
  const { onOpenChange } = useMenuContext();
  const { entry, exit } = useSelectedLocations();
  const isExitSingleRelay = exit && 'hostname' in exit;
  const isExitSelected = isLocationSelected(location.details, exit);

  const handleClick = React.useCallback(async () => {
    if (isExitSingleRelay && isExitSelected) {
      // Swap entry and exit location
      await setMultihop({ multihop, entryLocation: location.details, exitLocation: entry });
    } else {
      await selectEntryRelayLocation(location.details);
    }
    onOpenChange?.(false);
  }, [
    isExitSingleRelay,
    isExitSelected,
    onOpenChange,
    setMultihop,
    multihop,
    location.details,
    entry,
    selectEntryRelayLocation,
  ]);

  const isDaitaEnabledWithoutDirectOnly = useIsDaitaEnabledWithoutDirectOnly();
  const isExitSelectedWithoutMultihop = !multihop && isExitSelected;

  const disabled = isDaitaEnabledWithoutDirectOnly || isExitSelectedWithoutMultihop;

  return (
    <Menu.Option disabled={disabled} {...props}>
      <Menu.Option.Trigger onClick={handleClick}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="location-add" />
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Text for button that sets a location as entry relay.
              messages.gettext('Set as entry')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
