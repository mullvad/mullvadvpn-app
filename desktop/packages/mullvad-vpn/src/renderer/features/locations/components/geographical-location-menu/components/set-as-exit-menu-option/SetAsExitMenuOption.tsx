import React from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import { useMenuContext } from '../../../../../../lib/components/menu/MenuContext';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import { useMultihop } from '../../../../../multihop/hooks';
import { useRelayLocations, useSelectedLocations } from '../../../../hooks';
import type { GeographicalLocation } from '../../../../types';
import { isLocationSelected } from '../../../../utils';

export type SetAsExitMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
};

export function SetAsExitMenuOption({ location, ...props }: SetAsExitMenuOptionProps) {
  const { selectExitRelayLocation } = useRelayLocations();
  const { onOpenChange } = useMenuContext();
  const { multihop, setMultihop } = useMultihop();
  const { entry, exit } = useSelectedLocations();

  const isEntrySingleRelay = entry && 'hostname' in entry;
  const isEntrySelected = isLocationSelected(location.details, entry);

  const handleClick = React.useCallback(async () => {
    if (isEntrySingleRelay && isEntrySelected) {
      // Swap entry and exit location
      await setMultihop({
        multihop,
        entryLocation: exit,
        exitLocation: location.details,
      });
    } else {
      await selectExitRelayLocation(location.details);
    }
    onOpenChange?.(false);
  }, [
    exit,
    isEntrySelected,
    isEntrySingleRelay,
    location.details,
    multihop,
    onOpenChange,
    selectExitRelayLocation,
    setMultihop,
  ]);

  return (
    <Menu.Option {...props}>
      <Menu.Option.Trigger onClick={handleClick}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="location-add" />
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Text for button that sets a location as exit relay
              messages.gettext('Set as multihop exit')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
