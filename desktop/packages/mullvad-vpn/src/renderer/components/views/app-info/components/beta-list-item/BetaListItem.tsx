import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useVersionIsBeta } from '../../../../../redux/hooks';
import { SettingsListItem } from '../../../../settings-list-item';
import { BetaSwitch } from '../beta-switch';

export function BetaListItem() {
  const { isBeta } = useVersionIsBeta();

  const labelId = React.useId();
  const descriptionId = React.useId();

  return (
    <SettingsListItem disabled={isBeta}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <BetaSwitch labelId={labelId} descriptionId={descriptionId}>
            <BetaSwitch.Label variant="titleMedium">
              {
                // TRANSLATORS: Label for switch to toggle beta program.
                messages.pgettext('app-info-view', 'Beta program')
              }
            </BetaSwitch.Label>
            <BetaSwitch.Trigger>
              <BetaSwitch.Thumb />
            </BetaSwitch.Trigger>
          </BetaSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {isBeta
            ? // TRANSLATORS: Description for beta program switch when using a beta version.
              messages.pgettext(
                'app-info-view',
                'This option is unavailable while using a beta version.',
              )
            : // TRANSLATORS: Description for beta program switch.
              messages.pgettext(
                'app-info-view',
                'Enable to get notified when new beta versions of the app are released.',
              )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
