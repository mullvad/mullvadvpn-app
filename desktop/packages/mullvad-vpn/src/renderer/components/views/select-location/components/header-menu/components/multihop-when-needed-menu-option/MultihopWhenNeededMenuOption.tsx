import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useMultihop } from '../../../../../../../features/multihop/hooks';
import { Menu } from '../../../../../../../lib/components/menu';
import { useHeaderMenuContext } from '../../HeaderMenuContext';
import { useSetMultihopWhenNeeded } from '../../hooks';

export function MultihopWhenNeededMenuOption() {
  const { multihop } = useMultihop();
  const { onOpenChange } = useHeaderMenuContext();
  const setMultihopWhenNeeded = useSetMultihopWhenNeeded();

  const handleMultihopWhenNeeded = React.useCallback(async () => {
    await setMultihopWhenNeeded();
    onOpenChange?.(false);
  }, [onOpenChange, setMultihopWhenNeeded]);

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
