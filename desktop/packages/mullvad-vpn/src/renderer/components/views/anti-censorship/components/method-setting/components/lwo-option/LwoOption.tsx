import { strings } from '../../../../../../../../shared/constants';
import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { SettingsListbox } from '../../../../../../settings-listbox';

export function LwoOption() {
  return (
    <SettingsListbox.BaseOption value={ObfuscationType.lwo}>
      {strings.lwo}
    </SettingsListbox.BaseOption>
  );
}
