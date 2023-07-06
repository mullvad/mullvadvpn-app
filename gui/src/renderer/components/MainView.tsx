import { useEffect, useState } from 'react';

import { hasExpired } from '../../shared/account-expiry';
import { AuthFailedError, ErrorStateCause } from '../../shared/daemon-rpc-types';
import Connect from '../components/Connect';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import ExpiredAccountErrorView from './ExpiredAccountErrorView';

type ExpiryData = { show: false } | { show: true; expiry: string | undefined };

export default function MainView() {
  const { updateAccountData } = useAppContext();
  const history = useHistory();
  const accountExpiry = useSelector((state) => state.account.expiry);
  const accountHasExpired = accountExpiry !== undefined && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const tunnelState = useSelector((state) => state.connection.status);

  const [showAccountExpired, setShowAccountExpired] = useState<ExpiryData>(() =>
    isNewAccount || accountHasExpired ? { show: true, expiry: accountExpiry } : { show: false },
  );

  useEffect(() => {
    updateAccountData();
  }, []);

  useEffect(() => {
    if (
      (!showAccountExpired.show || showAccountExpired.expiry !== accountExpiry) &&
      (accountHasExpired ||
        (tunnelState.state === 'error' &&
          tunnelState.details.cause === ErrorStateCause.authFailed &&
          tunnelState.details.authFailedError === AuthFailedError.expiredAccount))
    ) {
      setShowAccountExpired({ show: true, expiry: accountExpiry });
    } else if (
      showAccountExpired.show &&
      accountExpiry &&
      accountExpiry !== showAccountExpired.expiry &&
      !accountHasExpired
    ) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired, tunnelState.state]);

  if (showAccountExpired.show) {
    return <ExpiredAccountErrorView />;
  } else {
    return <Connect />;
  }
}
