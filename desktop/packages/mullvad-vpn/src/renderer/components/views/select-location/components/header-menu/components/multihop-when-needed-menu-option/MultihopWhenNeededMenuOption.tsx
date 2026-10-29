import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationType } from '../../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../../features/multihop/hooks';
import { Menu } from '../../../../../../../lib/components/menu';
import { useSelectLocationViewContext } from '../../../../SelectLocationViewContext';
import { useHeaderMenuContext } from '../../HeaderMenuContext';
import { useResetLocationSelector } from '../../hooks';

export function MultihopWhenNeededMenuOption() {
  const { multihop, setMultihop } = useMultihop();
  const { onOpenChange } = useHeaderMenuContext();
  const { locationType, setLocationType } = useSelectLocationViewContext();
  const resetLocationSelector = useResetLocationSelector();

  const handleMultihopWhenNeeded = React.useCallback(async () => {
    await setMultihop({ multihop: 'when-needed' });
    onOpenChange?.(false);
    if (locationType === LocationType.entry || locationType === LocationType.entryAutomatic) {
      resetLocationSelector();
      setLocationType(LocationType.exit);
    }
  }, [locationType, onOpenChange, resetLocationSelector, setLocationType, setMultihop]);

  return (
    <Menu.Option>
      <Menu.Option.Trigger onClick={handleMultihopWhenNeeded}>
        <Menu.Option.Item>
          {multihop === 'when-needed' ? (
            <Menu.Option.Item.Icon icon="checkmark" />
          ) : (
            <Menu.Option.Item.Icon icon="placeholder" />
          )}
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Label for a menu option to change the Multihop mode setting
              // TRANSLATORS: option to "When needed".
              messages.pgettext('select-location-view', 'When needed')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
