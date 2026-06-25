import React from 'react';

import { MultihopMode } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';
import { SettingsListbox } from '../../../../components/settings-listbox';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useMultihop } from '../../hooks';

export function MultihopSetting() {
  const { multihop, setMultihop } = useMultihop();

  const normalRelaySettings = useNormalRelaySettings();
  const unavailable = normalRelaySettings === null;

  const handleValueChange = React.useCallback(
    async (multihop: MultihopMode) => {
      await setMultihop({ multihop });
    },
    [setMultihop],
  );

  return (
    <SettingsListbox anchorId="multihop-setting" value={multihop} onValueChange={handleValueChange}>
      <SettingsListbox.Header>
        <SettingsListbox.Header.Item>
          <SettingsListbox.Header.Item.Label>
            {
              // TRANSLATORS: Title for Multihop mode setting.
              messages.pgettext('wireguard-settings-view', 'Mode')
            }
          </SettingsListbox.Header.Item.Label>
        </SettingsListbox.Header.Item>
      </SettingsListbox.Header>
      <SettingsListbox.Options>
        <SettingsListbox.Options.BaseOption value="when-needed" disabled={unavailable}>
          {
            // TRANSLATORS: Label for the Multihop mode setting's option to use the multihop
            // TRANSLATORS: feature when it is needed.
            messages.pgettext('wireguard-settings-view', 'When needed')
          }
        </SettingsListbox.Options.BaseOption>
        <SettingsListbox.Options.BaseOption value="always" disabled={unavailable}>
          {
            // TRANSLATORS: Label for the Multihop mode setting's option to always use the multihop
            // TRANSLATORS: feature.
            messages.pgettext('wireguard-settings-view', 'Always')
          }
        </SettingsListbox.Options.BaseOption>
        <SettingsListbox.Options.BaseOption value="never" disabled={unavailable}>
          {
            // TRANSLATORS: Label for the Multihop mode setting's option to never use the multihop
            // TRANSLATORS: feature.
            messages.pgettext('wireguard-settings-view', 'Never')
          }
        </SettingsListbox.Options.BaseOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}
