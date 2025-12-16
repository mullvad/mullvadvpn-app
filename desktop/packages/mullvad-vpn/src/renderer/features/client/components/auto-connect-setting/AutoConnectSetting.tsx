import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { AutoConnectSwitch } from '../auto-connect-switch/AutoConnectSwitch';

export type AutoConnectSettingProps = Omit<ListItemProps, 'children'>;

export function AutoConnectSetting(props: AutoConnectSettingProps) {
  const descriptionId = React.useId();
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <AutoConnectSwitch>
            <AutoConnectSwitch.Label>
              {messages.pgettext('vpn-settings-view', 'Auto-connect')}
            </AutoConnectSwitch.Label>
            <AutoConnectSwitch.Trigger aria-describedby={descriptionId}>
              <AutoConnectSwitch.Thumb />
            </AutoConnectSwitch.Trigger>
          </AutoConnectSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {messages.pgettext(
            'vpn-settings-view',
            'Automatically connect to a server when the app launches.',
          )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
