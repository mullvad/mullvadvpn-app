import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { UnpinnedWindowSwitch } from '../unpinned-window-switch/UnpinnedWindowSwitch';

export type UnpinnedWindowSettingProps = Omit<ListItemProps, 'children'>;

export function UnpinnedWindowSetting(props: UnpinnedWindowSettingProps) {
  const descriptionId = React.useId();
  const messageLabel =
    process.platform === 'win32'
      ? // This line is here to prevent the following one to be moved up here by prettier
        // TRANSLATORS: Label for a setting to unpin the app window from the Windows "taskbar"
        messages.pgettext('user-interface-settings-view', 'Unpin app from taskbar')
      : // This line is here to prevent the following one to be moved up here by prettier
        // TRANSLATORS: Label for a setting to unpin the app window from the macOS "menu bar"
        messages.pgettext('user-interface-settings-view', 'Unpin app from menu bar');
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <UnpinnedWindowSwitch descriptionId={descriptionId}>
          <UnpinnedWindowSwitch.Label>{messageLabel}</UnpinnedWindowSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <UnpinnedWindowSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </UnpinnedWindowSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Footer.Text id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Enable to move the app around as a free-standing window.',
          )}
        </SettingsListItem.Footer.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
