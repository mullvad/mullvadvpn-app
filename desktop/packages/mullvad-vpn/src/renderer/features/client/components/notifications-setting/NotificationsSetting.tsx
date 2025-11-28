import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { NotificationsSwitch } from '../notifications-switch/NotificationsSwitch';

export function NotificationsSetting() {
  const descriptionId = React.useId();
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <NotificationsSwitch>
            <NotificationsSwitch.Label variant="titleMedium">
              {messages.pgettext('user-interface-settings-view', 'Notifications')}
            </NotificationsSwitch.Label>
            <NotificationsSwitch.Trigger aria-describedby={descriptionId}>
              <NotificationsSwitch.Thumb />
            </NotificationsSwitch.Trigger>
          </NotificationsSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Enable or disable system notifications. The critical notifications will always be displayed.',
          )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
