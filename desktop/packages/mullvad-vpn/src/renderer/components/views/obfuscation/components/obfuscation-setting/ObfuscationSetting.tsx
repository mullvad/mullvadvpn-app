import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { Constraint, ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useAppContext } from '../../../../../context';
import { Text } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';

export function formatPortForSubLabel(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}

export function ObfuscationSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  const obfuscationType = obfuscationSettings.selectedObfuscation;

  const selectObfuscationType = useCallback(
    async (value: ObfuscationType) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        selectedObfuscation: value,
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <SettingsListbox
      anchorId="obfuscation-setting"
      onValueChange={selectObfuscationType}
      value={obfuscationType}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard obfuscation selector.
              messages.pgettext('wireguard-settings-view', 'Obfuscation')
            }
          </SettingsListbox.Label>
          <InfoButton>
            <ModalMessage>
              {
                // TRANSLATORS: Describes what WireGuard obfuscation does, how it works and when
                // TRANSLATORS: it would be useful to enable it.
                messages.pgettext(
                  'wireguard-settings-view',
                  'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connection would be blocked.',
                )
              }
            </ModalMessage>
          </InfoButton>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={ObfuscationType.auto}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={ObfuscationType.lwo}>
          {strings.lwo}
        </SettingsListbox.BaseOption>
        <SettingsListbox.SplitOption value={ObfuscationType.shadowsocks}>
          <SettingsListbox.SplitOption.Item
            aria-description={messages.pgettext(
              'accessibility',
              'Use the right arrow key to focus the settings button.',
            )}>
            <FlexColumn>
              <SettingsListbox.SplitOption.Label>
                {messages.pgettext('wireguard-settings-view', 'Shadowsocks')}
              </SettingsListbox.SplitOption.Label>
              <Text variant="labelTinySemiBold" color="whiteAlpha60">
                {sprintf(subLabelTemplate, {
                  port: formatPortForSubLabel(obfuscationSettings.shadowsocksSettings.port),
                })}
              </Text>
            </FlexColumn>
          </SettingsListbox.SplitOption.Item>
          <SettingsListbox.SplitOption.NavigateButton
            to={RoutePath.shadowsocks}
            aria-label={messages.pgettext('accessibility', 'Shadowsocks settings')}
          />
        </SettingsListbox.SplitOption>
        <SettingsListbox.SplitOption value={ObfuscationType.udp2tcp}>
          <SettingsListbox.SplitOption.Item
            aria-description={messages.pgettext(
              'accessibility',
              'Use the right arrow key to focus the settings button.',
            )}>
            <FlexColumn>
              <SettingsListbox.SplitOption.Label>
                {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
              </SettingsListbox.SplitOption.Label>
              <Text variant="labelTinySemiBold" color="whiteAlpha60">
                {sprintf(subLabelTemplate, {
                  port: formatPortForSubLabel(obfuscationSettings.udp2tcpSettings.port),
                })}
              </Text>
            </FlexColumn>
          </SettingsListbox.SplitOption.Item>
          <SettingsListbox.SplitOption.NavigateButton
            to={RoutePath.udpOverTcp}
            aria-label={messages.pgettext('accessibility', 'UDP-over-TCP settings')}
          />
        </SettingsListbox.SplitOption>
        <SettingsListbox.BaseOption value={ObfuscationType.quic}>
          {strings.quic}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={ObfuscationType.off}>
          {messages.gettext('Off')}
        </SettingsListbox.BaseOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}
