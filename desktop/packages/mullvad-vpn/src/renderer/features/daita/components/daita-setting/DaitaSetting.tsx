import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { DaitaSwitch } from '../daita-switch';

export function DaitaSetting() {
  const descriptionId = React.useId();

  const relaySettings = useNormalRelaySettings();
  const disabled = relaySettings === undefined;

  return (
    <SettingsListItem anchorId="daita-enable-setting" disabled={disabled}>
      <SettingsListItem.Item>
        <DaitaSwitch descriptionId={descriptionId}>
          <DaitaSwitch.Label>{messages.gettext('Enable')}</DaitaSwitch.Label>
          <SettingsListItem.ActionGroup>
            <DaitaSwitch.Trigger>
              <DaitaSwitch.Thumb />
            </DaitaSwitch.Trigger>
          </SettingsListItem.ActionGroup>
        </DaitaSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
