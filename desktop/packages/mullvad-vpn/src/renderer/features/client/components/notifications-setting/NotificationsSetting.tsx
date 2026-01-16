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
        <NotificationsSwitch descriptionId={descriptionId}>
          <NotificationsSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Notifications')}
          </NotificationsSwitch.Label>
          <SettingsListItem.ActionGroup>
            <NotificationsSwitch.Thumb />
          </SettingsListItem.ActionGroup>
        </NotificationsSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.FooterText id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Enable or disable system notifications. The critical notifications will always be displayed.',
          )}
        </SettingsListItem.FooterText>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
