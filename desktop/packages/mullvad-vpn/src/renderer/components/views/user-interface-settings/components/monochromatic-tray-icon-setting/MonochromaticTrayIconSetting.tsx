import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { SettingsListItem } from '../../../../settings-list-item';
import { MonochromaticTrayIconSwitch } from './MonochromaticTrayIconSwitch';

export function MonochromaticTrayIconSetting() {
  const descriptionId = React.useId();
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <MonochromaticTrayIconSwitch>
            <MonochromaticTrayIconSwitch.Label variant="titleMedium">
              {messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon')}
            </MonochromaticTrayIconSwitch.Label>
            <MonochromaticTrayIconSwitch.Trigger aria-describedby={descriptionId}>
              <MonochromaticTrayIconSwitch.Thumb />
            </MonochromaticTrayIconSwitch.Trigger>
          </MonochromaticTrayIconSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Use a monochromatic tray icon instead of a colored one.',
          )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
