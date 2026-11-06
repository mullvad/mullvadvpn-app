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

export type SetAsExitMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
};

export function SetAsExitMenuOption({ location, ...props }: SetAsExitMenuOptionProps) {
  const { selectExitRelayLocation } = useRelayLocations();
  const { onOpenChange } = useMenuContext();
  const { multihop, setMultihop } = useMultihop();
  const { entryLocation } = useEntryLocation();
  const { exitLocation } = useExitLocation();
  const disabled = useDisabled(location);

  const isEntrySingleRelay = entryLocation !== 'any' && 'hostname' in entryLocation;
  const isEntrySelected = isLocationSelected(location.details, entryLocation);

  const handleClick = React.useCallback(async () => {
    if (isEntrySingleRelay && isEntrySelected) {
      // Swap entry and exit location
      await setMultihop({
        multihop,
        exit: exitLocation,
        entry: location.details,
      });
    } else {
      await selectExitRelayLocation(location.details);
    }
    onOpenChange?.(false);
  }, [
    exitLocation,
    isEntrySelected,
    isEntrySingleRelay,
    location.details,
    multihop,
    onOpenChange,
    selectExitRelayLocation,
    setMultihop,
  ]);

  return (
    <Menu.Option disabled={disabled} {...props}>
      <Menu.Option.Trigger onClick={handleClick}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="location-add" />
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Text for button that sets a location as exit relay
              messages.gettext('Set as exit')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
