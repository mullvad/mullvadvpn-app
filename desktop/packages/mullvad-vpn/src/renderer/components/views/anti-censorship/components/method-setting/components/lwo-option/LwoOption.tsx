import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../shared/routes';
import { Text } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { useSelector } from '../../../../../../../redux/store';
import { SettingsListbox } from '../../../../../../settings-listbox';
import { formatObfuscationPort } from '../../../../utils';

export function LwoOption() {
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);
  const port = obfuscationSettings.lwoSettings.port;

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('lwo-settings-view', 'Port: %(port)s');

  return (
    <SettingsListbox.Options.SplitOption value={ObfuscationType.lwo}>
      <SettingsListbox.Options.SplitOption.Item
        aria-description={messages.pgettext(
          'accessibility',
          'Use the right arrow key to focus the settings button.',
        )}>
        <FlexColumn>
          <SettingsListbox.Options.SplitOption.Label>
            {strings.lwo}
          </SettingsListbox.Options.SplitOption.Label>
          {port && (
            <Text variant="labelTinySemiBold" color="whiteAlpha60">
              {sprintf(subLabelTemplate, {
                port: formatObfuscationPort(port),
              })}
            </Text>
          )}
        </FlexColumn>
      </SettingsListbox.Options.SplitOption.Item>
      <SettingsListbox.Options.SplitOption.NavigateButton
        to={RoutePath.lwo}
        aria-label={sprintf(
          // TRANSLATORS: Text for screen readers to describe the LWO settings navigation button.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(lwo)s - will be replaced with LWO
          messages.pgettext('accessibility', '%(lwo)s settings'),
          { lwo: strings.lwo },
        )}
      />
    </SettingsListbox.Options.SplitOption>
  );
}
