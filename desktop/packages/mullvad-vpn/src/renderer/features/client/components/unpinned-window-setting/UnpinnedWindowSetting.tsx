import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { UnpinnedWindowSwitch } from '../unpinned-window-switch/UnpinnedWindowSwitch';

export type UnpinnedWindowSettingProps = Omit<ListItemProps, 'children'>;

export function UnpinnedWindowSetting(props: UnpinnedWindowSettingProps) {
  const descriptionId = React.useId();
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <UnpinnedWindowSwitch>
            <UnpinnedWindowSwitch.Label>
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
