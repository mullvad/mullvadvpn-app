import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { NotificationsSwitch } from '../notifications-switch/NotificationsSwitch';

export type NotificationsSettingProps = Omit<ListItemProps, 'children'>;

export function NotificationsSetting(props: NotificationsSettingProps) {
  const descriptionId = React.useId();
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <NotificationsSwitch>
            <NotificationsSwitch.Label>
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
