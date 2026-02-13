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
        <MonochromaticTrayIconSwitch descriptionId={descriptionId}>
          <MonochromaticTrayIconSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon')}
          </MonochromaticTrayIconSwitch.Label>
          <SettingsListItem.ActionGroup>
            <MonochromaticTrayIconSwitch.Input />
          </SettingsListItem.ActionGroup>
        </MonochromaticTrayIconSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.FooterText id={descriptionId}>
          {messages.pgettext(
            'user-interface-settings-view',
            'Use a monochromatic tray icon instead of a colored one.',
          )}
        </SettingsListItem.FooterText>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
