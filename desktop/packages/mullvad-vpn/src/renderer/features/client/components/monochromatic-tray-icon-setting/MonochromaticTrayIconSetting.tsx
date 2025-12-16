import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { MonochromaticTrayIconSwitch } from '../monochromatic-tray-icon-switch';

export type MonochromaticTrayIconSettingProps = Omit<ListItemProps, 'children'>;

export function MonochromaticTrayIconSetting(props: MonochromaticTrayIconSettingProps) {
  const descriptionId = React.useId();
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <MonochromaticTrayIconSwitch>
            <MonochromaticTrayIconSwitch.Label>
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
