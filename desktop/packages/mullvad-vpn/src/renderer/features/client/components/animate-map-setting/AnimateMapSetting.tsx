import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { AnimateMapSwitch } from '../animate-map-switch/AnimateMapSwitch';

export function AnimateMapSetting() {
  const descriptionId = React.useId();
  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Content>
          <AnimateMapSwitch>
            <AnimateMapSwitch.Label variant="titleMedium">
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
