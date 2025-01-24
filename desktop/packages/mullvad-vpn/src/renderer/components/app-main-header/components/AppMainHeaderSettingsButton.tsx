import { useCallback } from 'react';

import { messages } from '../../../../shared/gettext';
import { IconButtonProps, MainHeader } from '../../../lib/components';
import { transitions, useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';

export type MainHeaderSettingsButtonProps = Omit<IconButtonProps, 'icon'>;

export function AppMainHeaderSettingsButton(props: MainHeaderSettingsButtonProps) {
  const history = useHistory();

  const openSettings = useCallback(() => {
    if (!props.disabled) {
      history.push(RoutePath.settings, { transition: transitions.show });
    }
  }, [history, props.disabled]);

  return (
    <MainHeader.IconButton
      icon="settings-filled"
      onClick={openSettings}
      aria-label={messages.gettext('Settings')}
    />
  );
}
