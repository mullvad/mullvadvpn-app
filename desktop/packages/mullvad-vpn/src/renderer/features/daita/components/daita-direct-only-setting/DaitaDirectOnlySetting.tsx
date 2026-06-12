import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useDaitaEnabled } from '../../hooks';
import { DaitaDirectOnlySwitch } from '../daita-direct-only-switch';

export function DaitaDirectOnlySetting() {
  const { daitaEnabled } = useDaitaEnabled();
  const relaySettings = useNormalRelaySettings();

  const unavailable = relaySettings === undefined;
  const disabled = !daitaEnabled || unavailable;

  return (
    <SettingsListItem disabled={disabled}>
      <SettingsListItem.Item>
        <DaitaDirectOnlySwitch>
          <DaitaDirectOnlySwitch.Label>
            {messages.gettext('Direct only')}
          </DaitaDirectOnlySwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <Info>
              <Info.Button />
              <Info.Dialog>
                <Info.Dialog.Text>
                  {sprintf(
                    messages.pgettext(
                      'wireguard-settings-view',
                      'By enabling “%(directOnly)s” you will have to manually select a server that is %(daita)s-enabled. This can cause you to end up in a blocked state until you have selected a compatible server in the “Select location” view.',
                    ),
                    {
                      daita: strings.daita,
                      directOnly: messages.gettext('Direct only'),
                    },
                  )}
                </Info.Dialog.Text>
              </Info.Dialog>
            </Info>
            <DaitaDirectOnlySwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </DaitaDirectOnlySwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
