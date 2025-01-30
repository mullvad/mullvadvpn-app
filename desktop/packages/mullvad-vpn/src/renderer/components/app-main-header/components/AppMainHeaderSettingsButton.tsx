import { useCallback } from 'react';

import { messages } from '../../../../shared/gettext';
import { IconButton, IconButtonProps, MainHeader } from '../../../lib/components';
import { TransitionType, useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';

export type MainHeaderSettingsButtonProps = Omit<IconButtonProps, 'icon'>;

export function AppMainHeaderSettingsButton(props: MainHeaderSettingsButtonProps) {
  const history = useHistory();

  const openSettings = useCallback(() => {
    if (!props.disabled) {
      history.push(RoutePath.settings, { transition: TransitionType.show });
    }
  }, [history, props.disabled]);

  return (
    <MainHeader.IconButton onClick={openSettings} aria-label={messages.gettext('Settings')}>
      <IconButton.Icon icon="settings-filled" />{' '}
    </MainHeader.IconButton>
  );
}
