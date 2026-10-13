import React from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import { useMenuContext } from '../../../../../../lib/components/menu/MenuContext';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import { useMultihop } from '../../../../../multihop/hooks';
import { useEntryLocation, useExitLocation, useRelayLocations } from '../../../../hooks';
import type { GeographicalLocation } from '../../../../types';
import { isLocationSelected } from '../../../../utils';
import { useDisabled } from './hooks';

export type SetAsEntryMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
};

export function SetAsEntryMenuOption({ location, ...props }: SetAsEntryMenuOptionProps) {
  const { multihop, setMultihop } = useMultihop();
  const { selectEntryRelayLocation } = useRelayLocations();
  const { onOpenChange } = useMenuContext();
  const { entryLocation } = useEntryLocation();
  const { exitLocation } = useExitLocation();
  const isExitSingleRelay = exitLocation !== 'any' && 'hostname' in exitLocation;
  const isExitSelected = isLocationSelected(location.details, exitLocation);
  const disabled = useDisabled(location);

  const handleClick = React.useCallback(async () => {
    if (isExitSingleRelay && isExitSelected) {
      // Swap entry and exit location
      await setMultihop({ multihop, entry: location.details, exit: entryLocation });
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
    entryLocation,
    selectEntryRelayLocation,
  ]);

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
