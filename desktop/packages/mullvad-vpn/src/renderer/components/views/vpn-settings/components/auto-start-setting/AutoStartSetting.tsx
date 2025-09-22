import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function AutoStartSetting() {
  const autoStart = useSelector((state) => state.settings.autoStart);
  const { setAutoStart: setAutoStartImpl } = useAppContext();

  const setAutoStart = useCallback(
    async (autoStart: boolean) => {
      try {
        await setAutoStartImpl(autoStart);
      } catch (e) {
        const error = e as Error;
        log.error(`Cannot set auto-start: ${error.message}`);
      }
    },
    [setAutoStartImpl],
  );

  return (
    <SettingsToggleListItem checked={autoStart} onCheckedChange={setAutoStart}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Launch app on start-up')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
