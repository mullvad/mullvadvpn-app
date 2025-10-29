import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../shared/routes';
import { Text } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { useNormalRelaySettings } from '../../../../../../../lib/relay-settings-hooks';
import { SettingsListbox } from '../../../../../../settings-listbox';
import { formatRelayPort } from '../../../../utils';

export function WireguardPortOption() {
  const normalRelaySettings = useNormalRelaySettings();
  const port = normalRelaySettings?.wireguard?.port;

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  return (
    <SettingsListbox.SplitOption value={ObfuscationType.port}>
      <SettingsListbox.SplitOption.Item
        aria-description={messages.pgettext(
          'accessibility',
          'Use the right arrow key to focus the settings button.',
        )}>
        <FlexColumn>
          <SettingsListbox.SplitOption.Label>
            {sprintf(
              // TRANSLATORS: The label for the WireGuard port option.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - will be replaced with WireGuard
              messages.gettext('%(wireguard)s port'),
              { wireguard: strings.wireguard },
            )}
          </SettingsListbox.SplitOption.Label>
          {port && (
            <Text variant="labelTinySemiBold" color="whiteAlpha60">
              {sprintf(subLabelTemplate, {
                port: formatRelayPort(port),
              })}
            </Text>
          )}
        </FlexColumn>
      </SettingsListbox.SplitOption.Item>
      <SettingsListbox.SplitOption.NavigateButton
        to={RoutePath.wireguardPort}
        aria-label={sprintf(
          // TRANSLATORS: Text for screen readers to describe the WireGuard port settings navigation button.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(wireguard)s - will be replaced with WireGuard
          messages.pgettext('accessibility', '%(wireguard)s port settings'),
          { wireguard: strings.wireguard },
        )}
      />
    </SettingsListbox.SplitOption>
  );
}
