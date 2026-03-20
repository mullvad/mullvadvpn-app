import { strings } from '../../../../../../../../shared/constants';
import { ObfuscationType } from '../../../../../../../../shared/daemon-rpc-types';
import { SettingsListbox } from '../../../../../../settings-listbox';

export function QuicOption() {
  return (
    <SettingsListbox.Options.BaseOption value={ObfuscationType.quic}>
      {strings.quic}
    </SettingsListbox.Options.BaseOption>
  );
}
