import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { SettingsListbox } from '../../../../../../settings-listbox';

export function AutomaticOption() {
  return (
    <SettingsListbox.BaseOption value={ObfuscationType.auto}>
      {messages.gettext('Automatic')}
    </SettingsListbox.BaseOption>
  );
}
