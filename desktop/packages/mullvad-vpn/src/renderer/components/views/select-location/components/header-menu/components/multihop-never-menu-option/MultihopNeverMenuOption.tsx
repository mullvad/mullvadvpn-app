import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useMultihop } from '../../../../../../../features/multihop/hooks';
import { Menu } from '../../../../../../../lib/components/menu';
import { useHeaderMenuContext } from '../../HeaderMenuContext';

export function MultihopNeverMenuOption() {
  const { multihop, setMultihop } = useMultihop();
  const { onOpenChange } = useHeaderMenuContext();

  const handleMultihopNever = React.useCallback(async () => {
    await setMultihop({ multihop: 'never' });
    onOpenChange?.(false);
  }, [onOpenChange, setMultihop]);

  return (
    <Menu.Option>
      <Menu.Option.Trigger onClick={handleMultihopNever}>
        <Menu.Option.Item>
          {multihop === 'never' ? (
            <Menu.Option.Item.Icon icon="checkmark" />
          ) : (
            <Menu.Option.Item.Icon icon="placeholder" />
          )}
          <Menu.Option.Item.Label>
            {
              // TRANSLATORS: Label for a menu option to change the Multihop mode setting
              // TRANSLATORS: option to "Never".
              messages.pgettext('select-location-view', 'Never')
            }
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
