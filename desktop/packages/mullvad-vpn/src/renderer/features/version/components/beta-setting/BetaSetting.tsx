import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { useVersionIsBeta } from '../../../../redux/hooks';
import { BetaSwitch } from '../beta-switch';

export type BetaSettingProps = Omit<ListItemProps, 'children'>;

export function BetaSetting(props: BetaSettingProps) {
  const { isBeta } = useVersionIsBeta();

  const labelId = React.useId();
  const descriptionId = React.useId();

  return (
    <SettingsListItem disabled={isBeta} {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <BetaSwitch labelId={labelId} descriptionId={descriptionId}>
            <BetaSwitch.Label>
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
        <SettingsListItem.FooterText id={descriptionId}>
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
        </SettingsListItem.FooterText>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
