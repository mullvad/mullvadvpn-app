import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { isPlatform } from '../../../../utils';
import { MonochromaticTrayIconSwitch } from '../monochromatic-tray-icon-switch';

export type MonochromaticTrayIconSettingProps = Omit<ListItemProps, 'children'>;

export function MonochromaticTrayIconSetting(props: MonochromaticTrayIconSettingProps) {
  const descriptionId = React.useId();
  const label = isPlatform('darwin')
    ? messages.pgettext('user-interface-settings-view', 'Monochromatic menu bar icon')
    : messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon');

  const description = isPlatform('darwin')
    ? messages.pgettext(
        'user-interface-settings-view',
        'Use a monochromatic menu bar icon instead of a colored one.',
      )
    : messages.pgettext(
        'user-interface-settings-view',
        'Use a monochromatic tray icon instead of a colored one.',
      );

  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <MonochromaticTrayIconSwitch descriptionId={descriptionId}>
          <MonochromaticTrayIconSwitch.Label>{label}</MonochromaticTrayIconSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <MonochromaticTrayIconSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </MonochromaticTrayIconSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Footer.Text id={descriptionId}>
          {description}
        </SettingsListItem.Footer.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
