import { messages } from '../../../../../../../../shared/gettext';
import { SettingsListbox } from '../../../../../../settings-listbox';

export function AutomaticOption() {
  return (
    <SettingsListbox.BaseOption value={null}>
      {messages.gettext('Automatic')}
    </SettingsListbox.BaseOption>
  );
}
