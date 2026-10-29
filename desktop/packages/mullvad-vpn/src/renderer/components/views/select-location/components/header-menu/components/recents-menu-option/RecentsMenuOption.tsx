import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useRecents } from '../../../../../../../features/locations/hooks';
import { Menu } from '../../../../../../../lib/components/menu';
import { useHeaderMenuContext } from '../../HeaderMenuContext';

export function RecentsMenuOption() {
  const { onOpenChange, setDisableRecentsDialogOpen } = useHeaderMenuContext();
  const { hasRecents, setEnabledRecents } = useRecents();

  const openDisableRecentsDialog = React.useCallback(() => {
    setDisableRecentsDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange, setDisableRecentsDialogOpen]);

  const enableRecents = React.useCallback(async () => {
    onOpenChange?.(false);
    await setEnabledRecents(true);
  }, [onOpenChange, setEnabledRecents]);

  const handleClick = React.useCallback(() => {
    if (hasRecents) {
      openDisableRecentsDialog();
    } else {
      void enableRecents();
    }
  }, [enableRecents, hasRecents, openDisableRecentsDialog]);

  return (
    <Menu.Option>
      <Menu.Option.Trigger onClick={handleClick}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="history-remove" />
          <Menu.Option.Item.Label>
            {hasRecents
              ? // TRANSLATORS: Used in button to disable showing list of recent locations.
                messages.pgettext('select-location-view', 'Disable recents')
              : // TRANSLATORS: Used in button to enable showing list of recent locations.
                messages.pgettext('select-location-view', 'Enable recents')}
          </Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}
