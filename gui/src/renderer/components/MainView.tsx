import { useEffect, useState } from 'react';

import { hasExpired } from '../../shared/account-expiry';
import Connect from '../components/Connect';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import ExpiredAccountErrorView from './ExpiredAccountErrorView';

type ExpiryData = { show: false } | { show: true; expiry: string | undefined };

export default function MainView() {
  const history = useHistory();
  const accountExpiry = useSelector((state) => state.account.expiry);
  const accountHasExpired = accountExpiry !== undefined && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const [showAccountExpired, setShowAccountExpired] = useState<ExpiryData>(() =>
    isNewAccount || accountHasExpired ? { show: true, expiry: accountExpiry } : { show: false },
  );

  useEffect(() => {
    if (
      accountHasExpired &&
      (!showAccountExpired.show || showAccountExpired.expiry !== accountExpiry)
    ) {
      setShowAccountExpired({ show: true, expiry: accountExpiry });
    } else if (
      showAccountExpired.show &&
      accountExpiry !== showAccountExpired.expiry &&
      !accountHasExpired
    ) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired]);

  if (showAccountExpired.show) {
    return <ExpiredAccountErrorView />;
  } else {
    return <Connect />;
  }
}
