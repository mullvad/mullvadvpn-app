import { sprintf } from 'sprintf-js';

import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../shared/routes';
import { Text } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { useSelector } from '../../../../../../../redux/store';
import { SettingsListbox } from '../../../../../../settings-listbox';
import { formatObfuscationPort } from '../../../../utils';

export function UdpOverTcpOption() {
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  return (
    <SettingsListbox.Options.SplitOption value={ObfuscationType.udp2tcp}>
      <SettingsListbox.Options.SplitOption.Item
        aria-description={messages.pgettext(
          'accessibility',
          'Use the right arrow key to focus the settings button.',
        )}>
        <FlexColumn>
          <SettingsListbox.Options.SplitOption.Label>
            {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
          </SettingsListbox.Options.SplitOption.Label>

          <Text variant="labelTinySemiBold" color="whiteAlpha60">
            {sprintf(subLabelTemplate, {
              port: formatObfuscationPort(obfuscationSettings.udp2tcpSettings.port),
            })}
          </Text>
        </FlexColumn>
      </SettingsListbox.Options.SplitOption.Item>
      <SettingsListbox.Options.SplitOption.NavigateButton
        to={RoutePath.udpOverTcp}
        aria-label={messages.pgettext('accessibility', 'UDP-over-TCP settings')}
      />
    </SettingsListbox.Options.SplitOption>
  );
}
