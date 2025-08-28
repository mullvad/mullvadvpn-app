import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { useSelector } from '../../../../../redux/store';
import { ToggleListItem } from '../../../../toggle-list-item';

export function AutoStartSetting() {
  const autoStart = useSelector((state) => state.settings.autoStart);
  const { setAutoStart: setAutoStartImpl } = useAppContext();
  const scrollToAnchor = useScrollToListItem();

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
    <ToggleListItem
      animation={scrollToAnchor?.animation}
      checked={autoStart}
      onCheckedChange={setAutoStart}>
      <ToggleListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Launch app on start-up')}
      </ToggleListItem.Label>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
