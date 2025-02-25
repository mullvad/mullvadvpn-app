import { useCallback } from 'react';

import { messages } from '../../../../shared/gettext';
import { IconButton, IconButtonProps, MainHeader } from '../../../lib/components';
import { transitions, useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';
import { useSelector } from '../../../redux/store';

export type MainHeaderBarAccountButtonProps = Omit<IconButtonProps, 'icon'>;

export const AppMainHeaderBarAccountButton = (props: MainHeaderBarAccountButtonProps) => {
  const history = useHistory();
  const openAccount = useCallback(
    () => history.push(RoutePath.account, { transition: transitions.show }),
    [history],
  );

  const loggedIn = useSelector((state) => state.account.status.type === 'ok');
  if (!loggedIn) {
    return null;
  }

  return (
    <MainHeader.IconButton
      onClick={openAccount}
      data-testid="account-button"
      aria-label={messages.gettext('Account settings')}
      {...props}>
      <IconButton.Icon icon="account-circle" />
    </MainHeader.IconButton>
  );
};
