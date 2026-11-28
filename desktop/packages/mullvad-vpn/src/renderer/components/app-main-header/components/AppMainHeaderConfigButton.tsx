import { useCallback } from 'react';

import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { IconButton, IconButtonProps, MainHeader } from '../../../lib/components';
import { TransitionType, useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';

export type MainHeaderBarConfigButtonProps = Omit<IconButtonProps, 'icon'>;

export const AppMainHeaderBarConfigButton = (props: MainHeaderBarConfigButtonProps) => {
  const history = useHistory();
  const openConfig = useCallback(
    () => history.push(RoutePath.wireguardConfig, { transition: TransitionType.show }),
    [history],
  );

  const hasIdentity = useSelector((state) => state.identity.identity !== undefined);
  if (!hasIdentity) {
    return null;
  }

  return (
    <MainHeader.IconButton
      onClick={openConfig}
      data-testid="config-button"
      aria-label={messages.pgettext('wireguard-config-view', 'WireGuard config')}
      {...props}>
      <IconButton.Icon icon="account-circle" />
    </MainHeader.IconButton>
  );
};
