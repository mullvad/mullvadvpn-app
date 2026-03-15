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
        <UnpinnedWindowSwitch descriptionId={descriptionId}>
          <UnpinnedWindowSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Unpin app from taskbar')}
          </UnpinnedWindowSwitch.Label>
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
