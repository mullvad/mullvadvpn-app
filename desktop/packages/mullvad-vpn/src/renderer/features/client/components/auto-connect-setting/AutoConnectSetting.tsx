import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { AutoConnectSwitch } from '../auto-connect-switch/AutoConnectSwitch';

export function AutoConnectSetting() {
  const descriptionId = React.useId();
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <AutoConnectSwitch>
            <AutoConnectSwitch.Label variant="titleMedium">
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
