import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationType } from '../../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../../features/multihop/hooks';
import { Menu } from '../../../../../../../lib/components/menu';
import { useSelectLocationViewContext } from '../../../../SelectLocationViewContext';
import { useHeaderMenuContext } from '../../HeaderMenuContext';

export function MultihopAlwaysMenuOption() {
  const { multihop, setMultihop } = useMultihop();
  const { onOpenChange } = useHeaderMenuContext();
  const { locationType, setLocationType } = useSelectLocationViewContext();

  const handleMultihopAlways = React.useCallback(async () => {
    await setMultihop({ multihop: 'always' });
    onOpenChange?.(false);
    if (locationType === LocationType.entryAutomatic) {
      setLocationType(LocationType.entry);
    } else {
      setLocationType(LocationType.exit);
    }
  }, [locationType, onOpenChange, setLocationType, setMultihop]);

  return (
    <Menu.Option>
      <Menu.Option.Trigger onClick={handleMultihopAlways}>
        <Menu.Option.Item>
          {multihop === 'always' ? (
            <Menu.Option.Item.Icon icon="checkmark" />
          ) : (
            <Menu.Option.Item.Icon icon="placeholder" />
          )}
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Label for a menu option to change the Multihop mode setting
              // TRANSLATORS: option to "Always".
              messages.pgettext('select-location-view', 'Always')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
