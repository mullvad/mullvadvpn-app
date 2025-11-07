import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { useDaitaEnabled } from '../../../../../features/daita/hooks';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListItem } from '../../../../settings-list-item';
import { DaitaDirectOnlySwitch } from '../daita-direct-only-switch';

export function DaitaDirectOnlySetting() {
  const daitaEnabled = useDaitaEnabled();
  const relaySettings = useNormalRelaySettings();

  const unavailable = relaySettings === undefined;
  const disabled = !daitaEnabled || unavailable;

  return (
    <SettingsListItem disabled={disabled}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <DaitaDirectOnlySwitch>
            <DaitaDirectOnlySwitch.Label variant="titleMedium">
              {messages.gettext('Direct only')}
            </DaitaDirectOnlySwitch.Label>
            <SettingsListItem.Group>
              <InfoButton>
                <ModalMessage>
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
                </ModalMessage>
              </InfoButton>
              <DaitaDirectOnlySwitch.Trigger>
                <DaitaDirectOnlySwitch.Thumb />
              </DaitaDirectOnlySwitch.Trigger>
            </SettingsListItem.Group>
          </DaitaDirectOnlySwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}
