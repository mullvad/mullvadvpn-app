import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { AnimateMapSwitch } from '../animate-map-switch/AnimateMapSwitch';

export type AnimateMapSettingProps = Omit<ListItemProps, 'children'>;

export function AnimateMapSetting(props: AnimateMapSettingProps) {
  const descriptionId = React.useId();
  return (
    <ListItem {...props}>
      <ListItem.Item>
        <ListItem.Content>
          <AnimateMapSwitch>
            <AnimateMapSwitch.Label>
              {messages.pgettext('user-interface-settings-view', 'Animate map')}
            </AnimateMapSwitch.Label>
            <AnimateMapSwitch.Trigger aria-describedby={descriptionId}>
              <AnimateMapSwitch.Thumb />
            </AnimateMapSwitch.Trigger>
          </AnimateMapSwitch>
        </ListItem.Content>
      </ListItem.Item>
      <ListItem.Footer>
        <ListItem.Text id={descriptionId}>
          {messages.pgettext('user-interface-settings-view', 'Animate map movements.')}
        </ListItem.Text>
      </ListItem.Footer>
    </ListItem>
  );
}
