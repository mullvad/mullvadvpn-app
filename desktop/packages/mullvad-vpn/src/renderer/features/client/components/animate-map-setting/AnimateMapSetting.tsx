import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { AnimateMapSwitch } from '../animate-map-switch/AnimateMapSwitch';

export type AnimateMapSettingProps = Omit<ListItemProps, 'children'>;

export function AnimateMapSetting(props: AnimateMapSettingProps) {
  const descriptionId = React.useId();
  return (
    <ListItem {...props}>
      <ListItem.Item>
        <AnimateMapSwitch descriptionId={descriptionId}>
          <AnimateMapSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Animate map')}
          </AnimateMapSwitch.Label>
          <SettingsListItem.ActionGroup>
            <AnimateMapSwitch.Thumb />
          </SettingsListItem.ActionGroup>
        </AnimateMapSwitch>
      </ListItem.Item>
      <ListItem.Footer>
        <ListItem.FooterText id={descriptionId}>
          {messages.pgettext('user-interface-settings-view', 'Animate map movements.')}
        </ListItem.FooterText>
      </ListItem.Footer>
    </ListItem>
  );
}
