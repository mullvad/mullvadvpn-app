import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { UnpinnedWindowSwitch } from '../unpinned-window-switch/UnpinnedWindowSwitch';

export function UnpinnedWindowSetting() {
  const descriptionId = React.useId();
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <UnpinnedWindowSwitch>
            <UnpinnedWindowSwitch.Label variant="titleMedium">
              {messages.pgettext('user-interface-settings-view', 'Unpin app from taskbar')}
            </UnpinnedWindowSwitch.Label>
            <UnpinnedWindowSwitch.Trigger aria-describedby={descriptionId}>
              <UnpinnedWindowSwitch.Thumb />
            </UnpinnedWindowSwitch.Trigger>
          </UnpinnedWindowSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Enable to move the app around as a free-standing window.',
          )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
