import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { SettingsListItem } from '../../../../settings-list-item';
import { DaitaSwitch } from '../daita-switch';

export function DaitaSetting() {
  const descriptionId = React.useId();

  const relaySettings = useNormalRelaySettings();
  const disabled = relaySettings === undefined;

  return (
    <SettingsListItem anchorId="daita-enable-setting" disabled={disabled}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <DaitaSwitch descriptionId={descriptionId}>
            <DaitaSwitch.Label variant="titleMedium">
              {messages.gettext('Enable')}
            </DaitaSwitch.Label>
            <DaitaSwitch.Trigger>
              <DaitaSwitch.Thumb />
            </DaitaSwitch.Trigger>
          </DaitaSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
