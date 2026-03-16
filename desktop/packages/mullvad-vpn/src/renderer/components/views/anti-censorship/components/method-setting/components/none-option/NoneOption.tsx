import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { SettingsListbox } from '../../../../../../settings-listbox';

export function NoneOption() {
  return (
    <SettingsListbox.Options.BaseOption value={ObfuscationType.off}>
      {messages.gettext('None')}
    </SettingsListbox.Options.BaseOption>
  );
}
